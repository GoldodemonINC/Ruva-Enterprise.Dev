# AGENTS.md

## Build & Test

- **Windows GNU toolchain**: On `x86_64-pc-windows-gnu`, `windows-sys` crate needs `dlltool.exe` which isn't on PATH. Run `export PATH="/c/msys64/mingw64/bin:$PATH"` before `cargo test` or `cargo build`.
- **Test command**: `cd Ruva && export PATH="/c/msys64/mingw64/bin:$PATH" && cargo test 2>&1 | tail -30`

## Parser Quirks

- **Nested generics `>>`**: Lexer produces a single `Shr` token for `>>`, but parser needs two `Gt` tokens. The fix uses `generic_depth` counter + `split_gt_pending` flag — when splitting, `advance()` returns `Gt` but does NOT increment `pos`; the second `Gt` is returned via `split_gt_pending` on the next call.
- **`fn` type parameters require explicit return types**: `fn(&T)` fails to parse — must be `fn(&T) -> Unit`. The parser always expects `Arrow` after `RParen` in function types.
- **Top-level `let` is invalid**: Module-level variable declarations must use `const`, not `let`. The parser rejects `let` as a top-level item.

## Module Resolution

- **Core module test failures cascade**: When `stdlib/core/mod.ruva` has a parse error, both `test_resolve_core_module` and `test_resolve_program_inlines_core` fail. The error message (`Expected type, got Shr`) is misleading — it points to the token position but the root cause may be elsewhere (e.g., missing return type on a `fn` parameter earlier in the file).

## Architecture

- **`generate_cargo_toml` belongs in trait**: The method was implemented in `codegen.rs` but not declared in the `CodeGenerator` trait in `backend.rs`. Adding it as a default trait method fixed the compilation error. Backends that don't need it inherit the default.
