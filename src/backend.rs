use crate::ast::Program;

/// Trait that all code generation backends must implement.
///
/// Each backend transforms a Ruva AST into source code for a target language.
#[allow(dead_code)]
pub trait CodeGenerator {
    /// Generate source code from a Ruva AST program.
    fn generate(&mut self, program: &Program) -> String;

    /// The name of the target language (e.g., "rust", "zig", "python").
    fn target_name(&self) -> &str;

    /// The file extension for generated files (e.g., ".rs", ".zig", ".py").
    fn file_extension(&self) -> &str;

    /// Generate Cargo.toml content (only meaningful for Rust target).
    fn generate_cargo_toml(&mut self) -> String {
        "[package]\nname = \"ruva_program\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nanyhow = \"1\"\n".to_string()
    }
}

/// Supported compilation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
    Zig,
    Python,
    C,
    Cpp,
    Wasm,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Rust => write!(f, "rust"),
            Target::Zig => write!(f, "zig"),
            Target::Python => write!(f, "python"),
            Target::C => write!(f, "c"),
            Target::Cpp => write!(f, "cpp"),
            Target::Wasm => write!(f, "wasm"),
        }
    }
}

impl Target {
    pub fn file_extension(&self) -> &str {
        match self {
            Target::Rust => ".rs",
            Target::Zig => ".zig",
            Target::Python => ".py",
            Target::C => ".c",
            Target::Cpp => ".cpp",
            Target::Wasm => ".wat",
        }
    }
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(Target::Rust),
            "zig" | "zg" => Ok(Target::Zig),
            "python" | "py" => Ok(Target::Python),
            "c" => Ok(Target::C),
            "cpp" | "c++" | "cxx" => Ok(Target::Cpp),
            "wasm" | "wat" => Ok(Target::Wasm),
            _ => Err(format!(
                "Unknown target '{}'. Supported: rust, zig, python, c, cpp, wasm",
                s
            )),
        }
    }
}

/// Create a code generator for the given target.
pub fn create_generator(target: Target) -> Box<dyn CodeGenerator> {
    match target {
        Target::Rust => Box::new(crate::codegen::CodeGen::new()),
        Target::Zig => Box::new(crate::codegen_zig::ZigCodeGen::new()),
        Target::Python => Box::new(crate::codegen_python::PythonCodeGen::new()),
        Target::C => Box::new(crate::codegen_c::CCodeGen::new()),
        Target::Cpp => Box::new(crate::codegen_cpp::CppCodeGen::new()),
        Target::Wasm => Box::new(crate::codegen_wasm::WasmCodeGen::new()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_parsing() {
        assert_eq!("rust".parse::<Target>().unwrap(), Target::Rust);
        assert_eq!("zig".parse::<Target>().unwrap(), Target::Zig);
        assert_eq!("python".parse::<Target>().unwrap(), Target::Python);
        assert_eq!("rs".parse::<Target>().unwrap(), Target::Rust);
        assert_eq!("py".parse::<Target>().unwrap(), Target::Python);
    }

    #[test]
    fn test_target_display() {
        assert_eq!(format!("{}", Target::Rust), "rust");
        assert_eq!(format!("{}", Target::Zig), "zig");
        assert_eq!(format!("{}", Target::Python), "python");
    }

    #[test]
    fn test_target_extensions() {
        assert_eq!(Target::Rust.file_extension(), ".rs");
        assert_eq!(Target::Zig.file_extension(), ".zig");
        assert_eq!(Target::Python.file_extension(), ".py");
    }

    #[test]
    fn test_create_generator() {
        let gen = create_generator(Target::Rust);
        assert_eq!(gen.target_name(), "rust");

        let gen = create_generator(Target::Zig);
        assert_eq!(gen.target_name(), "zig");

        let gen = create_generator(Target::Python);
        assert_eq!(gen.target_name(), "python");
    }

    #[test]
    fn test_invalid_target() {
        assert!("java".parse::<Target>().is_err());
        assert!("csharp".parse::<Target>().is_err());
    }
}
