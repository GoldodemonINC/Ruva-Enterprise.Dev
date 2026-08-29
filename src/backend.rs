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
}

/// Supported compilation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
    Zig,
    Python,
    Java,
    CSharp,
    Go,
    Swift,
    Kotlin,
    TypeScript,
    JavaScript,
    Lua,
    Ruby,
    Php,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Rust => write!(f, "rust"),
            Target::Zig => write!(f, "zig"),
            Target::Python => write!(f, "python"),
            Target::Java => write!(f, "java"),
            Target::CSharp => write!(f, "csharp"),
            Target::Go => write!(f, "go"),
            Target::Swift => write!(f, "swift"),
            Target::Kotlin => write!(f, "kotlin"),
            Target::TypeScript => write!(f, "typescript"),
            Target::JavaScript => write!(f, "javascript"),
            Target::Lua => write!(f, "lua"),
            Target::Ruby => write!(f, "ruby"),
            Target::Php => write!(f, "php"),
        }
    }
}

impl Target {
    pub fn file_extension(&self) -> &str {
        match self {
            Target::Rust => ".rs",
            Target::Zig => ".zig",
            Target::Python => ".py",
            Target::Java => ".java",
            Target::CSharp => ".cs",
            Target::Go => ".go",
            Target::Swift => ".swift",
            Target::Kotlin => ".kt",
            Target::TypeScript => ".ts",
            Target::JavaScript => ".js",
            Target::Lua => ".lua",
            Target::Ruby => ".rb",
            Target::Php => ".php",
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
            "java" | "jv" => Ok(Target::Java),
            "csharp" | "cs" | "c#" => Ok(Target::CSharp),
            "go" | "golang" => Ok(Target::Go),
            "swift" => Ok(Target::Swift),
            "kotlin" | "kt" => Ok(Target::Kotlin),
            "typescript" | "ts" => Ok(Target::TypeScript),
            "javascript" | "js" => Ok(Target::JavaScript),
            "lua" => Ok(Target::Lua),
            "ruby" | "rb" => Ok(Target::Ruby),
            "php" => Ok(Target::Php),
            _ => Err(format!(
                "Unknown target '{}'. Supported: rust, zig, python, java, csharp, go, swift, kotlin, typescript, javascript, lua, ruby, php",
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
        Target::Java => Box::new(crate::codegen_java::JavaCodeGen::new()),
        Target::CSharp => Box::new(crate::codegen_csharp::CSharpCodeGen::new()),
        Target::Go => Box::new(crate::codegen_go::GoCodeGen::new()),
        Target::Swift => Box::new(crate::codegen_swift::SwiftCodeGen::new()),
        Target::Kotlin => Box::new(crate::codegen_kotlin::KotlinCodeGen::new()),
        Target::TypeScript => Box::new(crate::codegen_typescript::TypeScriptCodeGen::new()),
        Target::JavaScript => Box::new(crate::codegen_javascript::JavaScriptCodeGen::new()),
        Target::Lua => Box::new(crate::codegen_lua::LuaCodeGen::new()),
        Target::Ruby => Box::new(crate::codegen_ruby::RubyCodeGen::new()),
        Target::Php => Box::new(crate::codegen_php::PhpCodeGen::new()),
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
        assert_eq!("java".parse::<Target>().unwrap(), Target::Java);
        assert_eq!("csharp".parse::<Target>().unwrap(), Target::CSharp);
        assert_eq!("go".parse::<Target>().unwrap(), Target::Go);
        assert_eq!("swift".parse::<Target>().unwrap(), Target::Swift);
        assert_eq!("kotlin".parse::<Target>().unwrap(), Target::Kotlin);
        assert_eq!("typescript".parse::<Target>().unwrap(), Target::TypeScript);
        assert_eq!("javascript".parse::<Target>().unwrap(), Target::JavaScript);
        assert_eq!("lua".parse::<Target>().unwrap(), Target::Lua);
        assert_eq!("ruby".parse::<Target>().unwrap(), Target::Ruby);
        assert_eq!("php".parse::<Target>().unwrap(), Target::Php);
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

        let gen = create_generator(Target::Java);
        assert_eq!(gen.target_name(), "java");

        let gen = create_generator(Target::CSharp);
        assert_eq!(gen.target_name(), "csharp");

        let gen = create_generator(Target::Go);
        assert_eq!(gen.target_name(), "go");

        let gen = create_generator(Target::Swift);
        assert_eq!(gen.target_name(), "swift");

        let gen = create_generator(Target::Kotlin);
        assert_eq!(gen.target_name(), "kotlin");

        let gen = create_generator(Target::TypeScript);
        assert_eq!(gen.target_name(), "typescript");

        let gen = create_generator(Target::JavaScript);
        assert_eq!(gen.target_name(), "javascript");

        let gen = create_generator(Target::Lua);
        assert_eq!(gen.target_name(), "lua");

        let gen = create_generator(Target::Ruby);
        assert_eq!(gen.target_name(), "ruby");

        let gen = create_generator(Target::Php);
        assert_eq!(gen.target_name(), "php");
    }

    #[test]
    fn test_invalid_target() {
        assert!("cobol".parse::<Target>().is_err());
        assert!("fortran".parse::<Target>().is_err());
    }
}
