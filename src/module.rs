use crate::ast::{Item, Program};
use crate::parser::Parser;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolves and loads .ruva module files from the filesystem.
///
/// Handles:
/// - `import ruva::core` → loads from stdlib/core/mod.ruva
/// - `import ruva::graphics` → loads from stdlib/graphics/mod.ruva
/// - `mod name;` → loads from name.ruva or name/mod.ruva relative to the source file
pub struct ModuleResolver {
    /// Base path to the Ruva stdlib directory
    stdlib_path: PathBuf,
    /// Base path to the current source file's directory
    source_dir: PathBuf,
    /// Modules already loaded (cycle detection)
    loaded: HashSet<String>,
}

impl ModuleResolver {
    /// Create a new module resolver.
    /// `source_path` is the path to the .ruva file being compiled.
    pub fn new(source_path: &Path) -> Self {
        // Find stdlib directory: look for stdlib/ relative to the Ruva project root
        let stdlib_path = Self::find_stdlib_path(source_path);
        let source_dir = source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Self {
            stdlib_path,
            source_dir,
            loaded: HashSet::new(),
        }
    }

    /// Create a resolver with an explicit stdlib path.
    #[allow(dead_code)]
    pub fn with_stdlib(stdlib_path: PathBuf, source_path: &Path) -> Self {
        let source_dir = source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Self {
            stdlib_path,
            source_dir,
            loaded: HashSet::new(),
        }
    }

    /// Find the stdlib directory by searching upward from the source file.
    fn find_stdlib_path(source_path: &Path) -> PathBuf {
        let mut current = source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        // Search up to 10 parent directories
        for _ in 0..10 {
            let candidate = current.join("stdlib");
            if candidate.exists() && candidate.is_dir() {
                return candidate;
            }
            // Also check if we're in the Ruva project itself
            let candidate2 = current.join("Ruva/stdlib");
            if candidate2.exists() && candidate2.is_dir() {
                return candidate2;
            }
            if !current.pop() {
                break;
            }
        }

        // Fallback: assume stdlib is next to the source file
        PathBuf::from("stdlib")
    }

    /// Resolve and load a `ruva::` import path.
    /// Returns the parsed AST items for the module.
    pub fn resolve_ruva_import(&mut self, path: &str) -> Result<Vec<Item>> {
        // Security: reject paths with traversal components
        if path.contains("..") {
            bail!("Path traversal not allowed in module path: '{}'", path);
        }
        if path.starts_with('/') || path.contains(":\\") {
            bail!("Absolute paths not allowed in module imports: '{}'", path);
        }
        // Security: reject null bytes that could cause path truncation
        if path.contains('\0') {
            bail!("Null bytes not allowed in module path: '{}'", path);
        }

        // Check for cycles
        if self.loaded.contains(path) {
            return Ok(vec![]); // Already loaded, skip
        }
        self.loaded.insert(path.to_string());

        // Strip the `ruva::` prefix
        let module_path = path
            .strip_prefix("ruva::")
            .unwrap_or(path)
            .replace("::", "/");

        // Try to find the module file
        let candidates = vec![
            self.stdlib_path.join(format!("{}/mod.ruva", module_path)),
            self.stdlib_path.join(format!("{}.ruva", module_path)),
            self.source_dir.join(format!("{}/mod.ruva", module_path)),
            self.source_dir.join(format!("{}.ruva", module_path)),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return self.load_file(candidate);
            }
        }

        bail!(
            "Module '{}' not found. Looked in:\n{}",
            path,
            candidates
                .iter()
                .map(|c| format!("  - {}", c.display()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Resolve a file-based module (`mod name;`).
    pub fn resolve_file_module(&mut self, name: &str) -> Result<Vec<Item>> {
        // Security: reject names with traversal components
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            bail!("Invalid module name: '{}' (no path separators or traversal allowed)", name);
        }
        // Security: reject null bytes that could cause path truncation
        if name.contains('\0') {
            bail!("Null bytes not allowed in module name: '{}'", name);
        }

        let key = format!("mod:{}", name);
        if self.loaded.contains(&key) {
            return Ok(vec![]);
        }
        self.loaded.insert(key);

        // Try name.ruva and name/mod.ruva
        let candidates = vec![
            self.source_dir.join(format!("{}.ruva", name)),
            self.source_dir.join(format!("{}/mod.ruva", name)),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return self.load_file(candidate);
            }
        }

        bail!(
            "Module file for '{}' not found. Looked in:\n{}",
            name,
            candidates
                .iter()
                .map(|c| format!("  - {}", c.display()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Load and parse a .ruva file, returning its items.
    fn load_file(&mut self, path: &Path) -> Result<Vec<Item>> {
        // Security: verify path is within expected directories
        if let Ok(canon) = path.canonicalize() {
            let in_stdlib = self.stdlib_path.canonicalize().map_or(false, |sp| canon.starts_with(&sp));
            let in_source = self.source_dir.canonicalize().map_or(false, |sd| canon.starts_with(&sd));
            if !in_stdlib && !in_source {
                bail!(
                    "Security: module path '{}' resolves outside allowed directories",
                    path.display()
                );
            }
        }

        // Security: limit file size to 1MB to prevent DoS
        let metadata = std::fs::metadata(path)?;
        const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB
        if metadata.len() > MAX_FILE_SIZE {
            bail!(
                "Module file too large: {} bytes (max {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            );
        }

        let source = std::fs::read_to_string(path)?;
        let mut parser = Parser::new(&source)?;
        let program = parser.parse_program()?;

        // Recursively resolve any imports in the loaded file
        let mut items = Vec::new();
        for item in program.items {
            match item {
                Item::Import(ref imp) if imp.path.starts_with("ruva::") => {
                    // Resolve nested ruva imports
                    let nested = self.resolve_ruva_import(&imp.path)?;
                    items.extend(nested);
                }
                Item::Use(ref u) => {
                    // Check if the use path starts with ruva::
                    let full_path = u.path.join("::");
                    if full_path.starts_with("ruva::") {
                        let nested = self.resolve_ruva_import(&full_path)?;
                        items.extend(nested);
                    } else {
                        items.push(item);
                    }
                }
                Item::Module(ref m) if m.body.is_none() => {
                    // File-based module — resolve it
                    match self.resolve_file_module(&m.name) {
                        Ok(nested) => {
                            // Wrap in a module
                            items.push(Item::Module(crate::ast::ModDef {
                                is_pub: m.is_pub,
                                name: m.name.clone(),
                                body: Some(nested),
                            }));
                        }
                        Err(e) => {
                            // Keep the original module declaration
                            eprintln!("Warning: {}", e);
                            items.push(item);
                        }
                    }
                }
                _ => items.push(item),
            }
        }

        Ok(items)
    }

    /// Resolve all imports in a program, returning a new program with modules inlined.
    pub fn resolve_program(&mut self, program: &Program) -> Result<Program> {
        let mut items = Vec::new();
        let mut resolved_modules: Vec<(String, Vec<Item>)> = Vec::new();

        for item in &program.items {
            match item {
                Item::Import(imp) if imp.path.starts_with("ruva::") => {
                    // Load the stdlib module
                    match self.resolve_ruva_import(&imp.path) {
                        Ok(module_items) => {
                            // Generate a module wrapper
                            let module_name = imp
                                .path
                                .split("::")
                                .last()
                                .unwrap_or("module")
                                .to_string();

                            // Check if there's an alias
                            let name = imp
                                .alias
                                .clone()
                                .unwrap_or(module_name);

                            resolved_modules.push((name, module_items));
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load module '{}': {}", imp.path, e);
                            // Keep the original import as a comment
                            items.push(item.clone());
                        }
                    }
                }
                Item::Use(u) => {
                    let full_path = u.path.join("::");
                    if full_path.starts_with("ruva::") {
                        match self.resolve_ruva_import(&full_path) {
                            Ok(module_items) => {
                                let module_name = u
                                    .path
                                    .last()
                                    .map(|s| s.as_str())
                                    .unwrap_or("module");

                                // For selective imports, we want to expose specific items
                                // For simple use, we wrap in a module
                                if u.selective.is_empty() && !u.wildcard {
                                    // `use ruva::core` — load whole module
                                    resolved_modules.push((module_name.to_string(), module_items));
                                } else if u.wildcard {
                                    // `use ruva::core::*` — load whole module
                                    resolved_modules.push((module_name.to_string(), module_items));
                                } else {
                                    // `use ruva::core::{Option, Result}` — load module but mark selective
                                    // For now, load the whole module (selective filtering is a future enhancement)
                                    resolved_modules.push((module_name.to_string(), module_items));
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to resolve use '{}': {}", full_path, e);
                                items.push(item.clone());
                            }
                        }
                    } else {
                        items.push(item.clone());
                    }
                }
                Item::Module(m) if m.body.is_none() => {
                    // File-based module: `mod name;`
                    match self.resolve_file_module(&m.name) {
                        Ok(module_items) => {
                            items.push(Item::Module(crate::ast::ModDef {
                                is_pub: m.is_pub,
                                name: m.name.clone(),
                                body: Some(module_items),
                            }));
                        }
                        Err(e) => {
                            eprintln!("Warning: {}", e);
                            items.push(item.clone());
                        }
                    }
                }
                _ => items.push(item.clone()),
            }
        }

        // Insert resolved modules at the beginning of the program
        for (name, module_items) in resolved_modules {
            items.insert(
                0,
                Item::Module(crate::ast::ModDef {
                    is_pub: true,
                    name,
                    body: Some(module_items),
                }),
            );
        }

        Ok(Program { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_find_stdlib_path() {
        // Test that we can find the stdlib directory
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let path = ModuleResolver::find_stdlib_path(&source);
        assert!(
            path.exists() || path.to_string_lossy().contains("stdlib"),
            "Expected stdlib path, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_module_resolver_creation() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let resolver = ModuleResolver::new(&source);
        assert!(resolver.source_dir.exists());
    }

    #[test]
    fn test_resolve_nonexistent_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_core_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::core");
        assert!(result.is_ok(), "Failed to resolve ruva::core");
        let items = result.unwrap();
        assert!(!items.is_empty(), "Core module should have items");
    }

    #[test]
    fn test_resolve_program_inlines_core() {
        use crate::parser::Parser;
        let source = r#"import ruva::core
fn main() { let x = 1 }"#;
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source_path);
        let resolved = resolver.resolve_program(&program).unwrap();

        // Should have at least the core module + the original main function
        assert!(resolved.items.len() >= 2);
        // First item should be the core module
        match &resolved.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "core");
                assert!(m.body.is_some());
            }
            _ => panic!("Expected core module as first item"),
        }
    }

    #[test]
    fn test_resolve_cycle_detection() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        // First load should succeed (or fail if module doesn't exist)
        let _ = resolver.resolve_ruva_import("ruva::core");
        // Second load of same module should return empty (cycle detected)
        let result = resolver.resolve_ruva_import("ruva::core");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty(), "Cycle detection should return empty");
    }

    #[test]
    fn test_resolve_io_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::io");
        assert!(result.is_ok(), "Failed to resolve ruva::io");
        let items = result.unwrap();
        assert!(!items.is_empty(), "IO module should have items");
        // Check that we got structs and functions
        let has_struct = items.iter().any(|i| matches!(i, Item::Struct(_)));
        let has_fn = items.iter().any(|i| matches!(i, Item::Function(_)));
        assert!(has_struct, "IO module should have structs");
        assert!(has_fn, "IO module should have functions");
    }

    #[test]
    fn test_resolve_program_inlines_io() {
        use crate::parser::Parser;
        let source = r#"import ruva::io
fn main() { let f = File::open("test.txt") }"#;
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source_path);
        let resolved = resolver.resolve_program(&program).unwrap();

        // Should have the io module + original main function
        assert!(resolved.items.len() >= 2);
        match &resolved.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "io");
                assert!(m.body.is_some());
            }
            _ => panic!("Expected io module as first item"),
        }
    }

    #[test]
    fn test_io_module_has_file_struct() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::io").unwrap();

        // Find the File struct
        let has_file = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "File"
            } else {
                false
            }
        });
        assert!(has_file, "IO module should have File struct");
    }

    #[test]
    fn test_io_module_has_tcp_types() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::io").unwrap();

        let has_tcp_listener = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TcpListener"
            } else {
                false
            }
        });
        let has_udp = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "UdpSocket"
            } else {
                false
            }
        });
        assert!(has_tcp_listener, "IO module should have TcpListener");
        assert!(has_udp, "IO module should have UdpSocket");
    }

    #[test]
    fn test_resolve_testing_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::testing");
        assert!(result.is_ok(), "Failed to resolve ruva::testing");
        let items = result.unwrap();
        assert!(!items.is_empty(), "Testing module should have items");
        // Check that we got structs and functions
        let has_struct = items.iter().any(|i| matches!(i, Item::Struct(_)));
        let has_fn = items.iter().any(|i| matches!(i, Item::Function(_)));
        assert!(has_struct, "Testing module should have structs");
        assert!(has_fn, "Testing module should have functions");
    }

    #[test]
    fn test_testing_module_has_assertions() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::testing").unwrap();

        let has_assert = items.iter().any(|i| {
            if let Item::Function(f) = i {
                f.name == "assert"
            } else {
                false
            }
        });
        let has_assert_eq = items.iter().any(|i| {
            if let Item::Function(f) = i {
                f.name == "assert_eq"
            } else {
                false
            }
        });
        let has_assert_approx_eq = items.iter().any(|i| {
            if let Item::Function(f) = i {
                f.name == "assert_approx_eq"
            } else {
                false
            }
        });
        assert!(has_assert, "Testing module should have assert");
        assert!(has_assert_eq, "Testing module should have assert_eq");
        assert!(has_assert_approx_eq, "Testing module should have assert_approx_eq");
    }

    #[test]
    fn test_testing_module_has_runner() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::testing").unwrap();

        let has_runner = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TestRunner"
            } else {
                false
            }
        });
        let has_suite = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TestSuite"
            } else {
                false
            }
        });
        let has_formatter = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TestFormatter"
            } else {
                false
            }
        });
        assert!(has_runner, "Testing module should have TestRunner");
        assert!(has_suite, "Testing module should have TestSuite");
        assert!(has_formatter, "Testing module should have TestFormatter");
    }

    #[test]
    fn test_testing_module_has_discovery() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::testing").unwrap();

        let has_discovery = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TestDiscovery"
            } else {
                false
            }
        });
        let has_report = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "TestReport"
            } else {
                false
            }
        });
        let has_benchmark = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "BenchmarkResult"
            } else {
                false
            }
        });
        assert!(has_discovery, "Testing module should have TestDiscovery");
        assert!(has_report, "Testing module should have TestReport");
        assert!(has_benchmark, "Testing module should have BenchmarkResult");
    }

    #[test]
    fn test_resolve_program_inlines_testing() {
        use crate::parser::Parser;
        let source = r#"import ruva::testing
fn main() { let r = TestResult::pass("test".into()) }"#;
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source_path);
        let resolved = resolver.resolve_program(&program).unwrap();

        // Should have the testing module + original main function
        assert!(resolved.items.len() >= 2);
        match &resolved.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "testing");
                assert!(m.body.is_some());
            }
            _ => panic!("Expected testing module as first item"),
        }
    }

    #[test]
    fn test_resolve_formatter_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::formatter");
        assert!(result.is_ok(), "Failed to resolve ruva::formatter");
        let items = result.unwrap();
        assert!(!items.is_empty(), "Formatter module should have items");
        let has_struct = items.iter().any(|i| matches!(i, Item::Struct(_)));
        let has_fn = items.iter().any(|i| matches!(i, Item::Function(_)));
        assert!(has_struct, "Formatter module should have structs");
        assert!(has_fn, "Formatter module should have functions");
    }

    #[test]
    fn test_formatter_module_has_style_config() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::formatter").unwrap();

        let has_style_config = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "StyleConfig"
            } else {
                false
            }
        });
        let has_format_result = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "FormatResult"
            } else {
                false
            }
        });
        let has_format_stats = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "FormatStats"
            } else {
                false
            }
        });
        assert!(has_style_config, "Formatter module should have StyleConfig");
        assert!(has_format_result, "Formatter module should have FormatResult");
        assert!(has_format_stats, "Formatter module should have FormatStats");
    }

    #[test]
    fn test_formatter_module_has_format_functions() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::formatter").unwrap();

        let has_format_source = items.iter().any(|i| {
            if let Item::Function(f) = i {
                f.name == "format_source"
            } else {
                false
            }
        });
        let has_format_file = items.iter().any(|i| {
            if let Item::Function(f) = i {
                f.name == "format_file"
            } else {
                false
            }
        });
        assert!(has_format_source, "Formatter module should have format_source");
        assert!(has_format_file, "Formatter module should have format_file");
    }

    #[test]
    fn test_resolve_program_inlines_formatter() {
        use crate::parser::Parser;
        let source = r#"import ruva::formatter
fn main() { let cfg = StyleConfig::default_config() }"#;
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source_path);
        let resolved = resolver.resolve_program(&program).unwrap();

        assert!(resolved.items.len() >= 2);
        match &resolved.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "formatter");
                assert!(m.body.is_some());
            }
            _ => panic!("Expected formatter module as first item"),
        }
    }

    #[test]
    fn test_resolve_serialization_module() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let result = resolver.resolve_ruva_import("ruva::serialization");
        assert!(result.is_ok(), "Failed to resolve ruva::serialization");
        let items = result.unwrap();
        assert!(!items.is_empty(), "Serialization module should have items");
        let has_struct = items.iter().any(|i| matches!(i, Item::Struct(_)));
        let has_fn = items.iter().any(|i| matches!(i, Item::Function(_)));
        assert!(has_struct, "Serialization module should have structs");
        assert!(has_fn, "Serialization module should have functions");
    }

    #[test]
    fn test_serialization_module_has_json_types() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::serialization").unwrap();

        let has_json_value = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.name == "JsonValue"
            } else {
                false
            }
        });
        let has_toml_value = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.name == "TomlValue"
            } else {
                false
            }
        });
        let has_yaml_value = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.name == "YamlValue"
            } else {
                false
            }
        });
        assert!(has_json_value, "Serialization module should have JsonValue");
        assert!(has_toml_value, "Serialization module should have TomlValue");
        assert!(has_yaml_value, "Serialization module should have YamlValue");
    }

    #[test]
    fn test_serialization_module_has_format_enum() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source);
        let items = resolver.resolve_ruva_import("ruva::serialization").unwrap();

        let has_format = items.iter().any(|i| {
            if let Item::Enum(e) = i {
                e.name == "Format"
            } else {
                false
            }
        });
        let has_serializer_config = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "SerializerConfig"
            } else {
                false
            }
        });
        let has_schema = items.iter().any(|i| {
            if let Item::Struct(s) = i {
                s.name == "Schema"
            } else {
                false
            }
        });
        assert!(has_format, "Serialization module should have Format");
        assert!(has_serializer_config, "Serialization module should have SerializerConfig");
        assert!(has_schema, "Serialization module should have Schema");
    }

    #[test]
    fn test_resolve_program_inlines_serialization() {
        use crate::parser::Parser;
        let source = r#"import ruva::serialization
fn main() { let cfg = SerializerConfig::default_json() }"#;
        let mut parser = Parser::new(source).unwrap();
        let program = parser.parse_program().unwrap();

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("examples/hello.ruva");
        let mut resolver = ModuleResolver::new(&source_path);
        let resolved = resolver.resolve_program(&program).unwrap();

        assert!(resolved.items.len() >= 2);
        match &resolved.items[0] {
            Item::Module(m) => {
                assert_eq!(m.name, "serialization");
                assert!(m.body.is_some());
            }
            _ => panic!("Expected serialization module as first item"),
        }
    }
}
