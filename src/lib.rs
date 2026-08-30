// Ruva language library — exposes compiler internals for integration tests and benchmarks.
// The binary crate (main.rs) also declares these modules; Cargo compiles both independently.

pub mod ast;
pub mod backend;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod typecheck;
pub mod module;
pub mod features;
pub mod vm;
pub mod colors;
pub mod debug;