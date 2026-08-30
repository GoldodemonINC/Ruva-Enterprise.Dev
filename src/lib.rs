// Ruva language library — exposes transpiler internals for integration tests and benchmarks.
// The binary crate (main.rs) also declares these modules; Cargo compiles both independently.

pub mod ast;
pub mod backend;
pub mod codegen;
pub mod codegen_python;
pub mod codegen_zig;
pub mod codegen_java;
pub mod codegen_csharp;
pub mod codegen_go;
pub mod codegen_swift;
pub mod codegen_kotlin;
pub mod codegen_typescript;
pub mod codegen_javascript;
pub mod codegen_lua;
pub mod codegen_ruby;
pub mod codegen_php;
pub mod lexer;
pub mod parser;
pub mod typecheck;
pub mod module;
pub mod features;
