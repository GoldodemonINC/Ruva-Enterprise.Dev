use crate::ast::Program;

/// Trait that all code generation backends must implement.
///
/// Each backend transforms a Ruva AST into source code for a target language.
pub trait CodeGenerator {
    /// Generate source code from a Ruva AST program.
    fn generate(&mut self, program: &Program) -> String;

    /// The name of the target language (e.g., "rust").
    fn target_name(&self) -> &str;

    /// The file extension for generated files (e.g., ".rs").
    fn file_extension(&self) -> &str;
}

/// Supported compilation targets. Ruva compiles to native via the Rust backend
/// or runs directly on its own bytecode VM; the historical multi-language
/// transpiler backends have been removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rust")
    }
}

impl Target {
    pub fn file_extension(&self) -> &str {
        ".rs"
    }
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(Target::Rust),
            _ => Err(format!(
                "Unknown target '{}'. Supported: rust",
                s
            )),
        }
    }
}

/// Create a code generator for the given target.
pub fn create_generator(_target: Target) -> Box<dyn CodeGenerator> {
    Box::new(crate::codegen::CodeGen::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_parsing() {
        assert_eq!("rust".parse::<Target>().unwrap(), Target::Rust);
        assert_eq!("rs".parse::<Target>().unwrap(), Target::Rust);
    }

    #[test]
    fn test_target_display() {
        assert_eq!(format!("{}", Target::Rust), "rust");
    }

    #[test]
    fn test_target_extensions() {
        assert_eq!(Target::Rust.file_extension(), ".rs");
    }

    #[test]
    fn test_create_generator() {
        let gen = create_generator(Target::Rust);
        assert_eq!(gen.target_name(), "rust");
        assert_eq!(gen.file_extension(), ".rs");
    }

    #[test]
    fn test_invalid_target() {
        assert!("cobol".parse::<Target>().is_err());
        assert!("zig".parse::<Target>().is_err());
        assert!("python".parse::<Target>().is_err());
    }
}