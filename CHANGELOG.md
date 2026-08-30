# Changelog

All notable changes to Ruva will be documented in this file.

## [0.9.0] — 2026-08-27

### Added
- **LSP (Language Server Protocol)** (`src/lsp.rs`) — Full language server implementation:
  - Text document synchronization (open, change, close)
  - Hover information (functions, structs, enums, classes, keywords)
  - Go-to-definition for all symbol types
  - Completion suggestions (keywords + document symbols with filtering)
  - Document symbols (outline view)
  - Diagnostics (parse errors + type checker integration)
  - JSON-RPC parser/serializer (zero external dependencies)
  - `ruva lsp` CLI command to start the language server
  - 29 LSP-specific unit tests
- **LSP Demo Example** (`examples/lsp_demo.ruva`) — Demonstrates all LSP features

## [0.8.0] — 2026-08-27

### Added
- **Serialization Standard Library** (`stdlib/serialization/`) — JSON, TOML, YAML parsing and conversion:
  - Format enum: `JSON`, `TOML`, `YAML`, `CSV`, `XML`, `Binary` with `extension()`, `mime_type()`, `from_extension`
  - JSON types: `JsonValue` — Null, Bool, Int, Float, Str, Array, Object with type checks and accessors
  - TOML types: `TomlValue` — Bool, Int, Float, Str, Datetime, Array, Table with type checks
  - YAML types: `YamlValue` — Null, Bool, Int, Float, Str, Seq, Map with type checks
  - Serializer config: `SerializerConfig` with `default_json/toml/yaml`, `compact_json`
  - Deserializer config: `DeserializerConfig` with `default_json/toml/yaml`, `lenient_json`
  - Serializer/Deserializer results: `SerializerResult`, `DeserializerResult`
  - Schema validation: `Schema`, `SchemaField` with `add_field`, `validate`
  - Encoding: `encode`, `encode_json/toml/yaml`, `encode_json_pretty`, `encode_to_string/file`
  - Decoding: `decode`, `decode_json/toml/yaml`, `decode_from_string/file`
  - Format detection: `detect_format`, `detect_format_from_path/extension/content`
  - Conversion: `convert`, `convert_json_to_toml/yaml`, `convert_toml_to_json/yaml`, `convert_yaml_to_json/toml`
  - Validation: `validate_json/toml/yaml`, `validate_against_schema`
  - Query/path: `query`, `query_int/string/bool`, `set_path`, `remove_path`, `has_path`
  - Merge/diff: `merge_json`, `deep_merge`, `patch`, `diff`, `JsonDiff`
  - Transform: `transform_keys`, `flatten`, `unflatten`, `select_keys`, `omit_keys`
  - Builders: `JsonBuilder`, `TomlBuilder`, `YamlBuilder`
  - Utilities: `escape_json/yaml_string`, `indent_json`, `minify_json`, `sort_json_keys`
- **Serialization demo example** (`serialization_demo.ruva`) demonstrating all serialization APIs
- **4 new tests** for serialization module resolution (68 total)
- All 9 stdlib modules parse and type-check cleanly

### Changed
- Renamed `get_indent_string` to `make_indent_string` (formatter module, v0.7.0)
- All 28 examples pass type-checking

## [0.7.0] — 2026-08-27

### Added
- **Formatter Standard Library** (`stdlib/formatter/`) — code formatting rules, style config, utilities:
  - Style configs: `StyleConfig` with `default_config`, `compact`, `rustfmt`, `python_style`, `zig_style`
  - Format options: `FormatOptions` with `check_only`, `dry_run`, `verbose`
  - Format results: `FormatResult` with `success`, `error`, `is_changed`
  - Format stats: `FormatStats` with `add_result`, `print_stats`, `is_all_formatted`
  - Format rules: `FormatRule` with `new`, `with_priority`, `disabled`, `enable/disable`
  - Format context: `FormatContext` with `indent/dedent`, enter/exit for functions, structs, impls
  - Format errors: `FormatError` with file, line, column, message, rule
  - Format diff: `FormatDiff`, `FormatHunk` for tracking changes
  - Formatting functions: `format_source`, `format_file`, `format_directory`, `check_format`
  - Indentation: `indent_line`, `indent_block`, `dedent_line`, `strip_indentation`
  - Line formatting: `trim_trailing_whitespace`, `ensure_trailing_newline`, `normalize_blank_lines`
  - Spacing: `normalize_spacing`, `ensure_space_after_keyword`, `normalize_operator_spacing`
  - Brace formatting: `normalize_braces`, `move_brace_to_next_line/same_line`
  - Import formatting: `sort_imports`, `group_imports`, `merge_duplicate_imports`
  - Trailing commas: `add_trailing_commas`, `remove_trailing_commas`
  - Line wrapping: `wrap_long_lines`, `break_long_function_call/sig/chain`
  - Match/pattern: `format_match_arms`, `format_pattern`, `normalize_arrow_spacing`
  - Comments: `align_comments`, `ensure_space_before_comment`, `format_doc_comments`
  - Config loading: `load_config`, `save_config`, `find_config_file`
- **`ruva fmt` CLI command** — format .ruva files in-place:
  - `ruva fmt <file>` — format a single file
  - `ruva fmt <dir>` — format all .ruva files in directory
  - `ruva fmt --check` — check only, don't modify (CI-friendly)
  - `ruva fmt --dry-run` — show what would change
  - `ruva fmt --verbose` — verbose output
  - Handles: trailing whitespace, blank line normalization, newline at EOF
- **Formatting demo example** (`formatting_demo.ruva`) demonstrating all formatter APIs
- **4 new tests** for formatter module resolution (64 total)
- All 8 stdlib modules parse and type-check cleanly

### Changed
- Renamed `get_indent_string` to `make_indent_string` to avoid method conflict
- All 27 examples pass type-checking

## [0.6.0] — 2026-08-27

### Added
- **Testing Standard Library** (`stdlib/testing/`) — test runner, assertions, discovery, formatting:
  - Assertions: `assert`, `assert_msg`, `assert_eq/ne`, `assert_true/false`, `assert_null/not_null`
  - Numeric: `assert_approx_eq/ne`, `assert_gt/gte/lt/lte`
  - String: `assert_str_eq/ne`, `assert_str_contains`, `assert_str_starts_with/ends_with`
  - Length: `assert_len`, `assert_empty`, `assert_not_empty`
  - Results/Options: `assert_result_ok/err`, `assert_option_some/none`
  - Test Runner: `TestRunner` with `run_assertion`, `run_test`, `print_results`, `print_failures`
  - Test Suite: `TestSuite` with `add_pass/fail`, `print_summary`, `get_failures`
  - Test Stats: `TestStats` with `pass_rate`, `merge`, `print_stats`
  - Test Discovery: `TestDiscovery` with `register_test`, `filter_by_category`
  - Test Formatter: `TestFormatter` with `format_pass/fail/skip`, `format_duration`
  - Test Report: `TestReport` with `add_result`, `print_report`
  - Benchmark: `BenchmarkResult` with `throughput`, `print_benchmark`
  - Config: `TestConfig` with `verbose`, `quiet`, `with_filter`
  - Test Case: `TestCase` with `new`, `with_category`, `should_fail`
  - Helpers: `run_test_group`, `print_pass/fail/skip`, `print_section`
- **Testing demo example** (`testing_stdlib.ruva`) demonstrating all testing APIs
- **5 new tests** for testing module resolution (60 total)
- All 7 stdlib modules parse and type-check cleanly

### Changed
- Renamed `test` variable to `tc` to avoid keyword conflict in `run_test_group`
- All 26 examples pass type-checking

## [0.5.0] — 2026-08-27

### Added
- **IO Standard Library** (`stdlib/io/`) — file operations, paths, directories, network, process:
  - File ops: `File::open`, `File::create`, `File::append`, `read_file`, `write_file`, `append_file`, `copy_file`, `remove_file`
  - File handle API: `file_read_all`, `file_write_all`, `file_read_line`, `file_flush`, `seek`, `metadata`
  - Path utilities: `PathBuf` struct with `join`, `parent`, `file_name`, `extension`, `exists`, `is_absolute`
  - Directory ops: `read_dir`, `create_dir`, `create_dir_all`, `remove_dir_all`, `dir_exists`, `walk_dir`, `find_files`
  - Stdout/Stderr/Stdin structs with `write`, `write_line`, `read_line`
  - TCP sockets: `TcpStream`, `TcpListener` with `connect`, `bind`, `accept`, `read`, `write`
  - UDP sockets: `UdpSocket` with `bind`, `send_to`, `recv_from`
  - Environment: `env_var`, `set_env_var`, `env_vars`
  - Process: `Command` struct with `arg`, `output`, `spawn`
  - Temp files: `TempFile`, `TempDir`
  - File watcher: `FileWatcher` stubs
- **IO demo example** (`io_demo.ruva`) demonstrating all IO APIs
- **5 new tests** for IO module resolution (55 total)
- All 6 stdlib modules parse and type-check cleanly

### Changed
- Method names prefixed to avoid cross-type conflicts (e.g. `file_read_all`, `tcp_write_all`)
- All 25 examples pass type-checking

## [0.4.0] — 2026-08-27

### Added
- **Standard Library** (`stdlib/`) — real functional implementations across all modules:
  - `core` — Option, Result, Vec, HashMap, Pair, Rng, math ops, string utils (500+ LOC)
  - `graphics` — OpenGL, Vulkan, DirectX 11/12 bindings
  - `browser` — WebAssembly, DOM, Canvas, WebGL, Fetch, WebSocket APIs
  - `video` — Video/Audio codec, encoder/decoder, muxer/demuxer, filters
  - `anticheat` — Memory protection, process info, anti-debug, anti-tamper, crypto
- **Module resolver** (`module.rs`) — loads .ruva files from filesystem at transpile time
  - `import ruva::core` → inlines stdlib/core/mod.ruva
  - `import ruva::graphics` → inlines stdlib/graphics/mod.ruva
  - Recursive resolution with cycle detection
- **Generic enums** — `enum Option<T>`, `enum Result<T, E>` now parse correctly
- **Enhanced type checker** — registers functions from loaded modules into global scope
  - Nested module items also registered (for `mod filter { ... }` inside stdlib)
- **stdlib_demo.ruva** — example demonstrating stdlib usage
- **3 new tests** for module resolution (51 total)
- All 5 stdlib modules parse and type-check cleanly

### Changed
- `check_file()` now resolves modules before type-checking
- `transpile()` now accepts source_path for module resolution
- `cmd_compile()` resolves modules before codegen
- Type checker registers unqualified function names from module bodies

## [0.3.0] — 2026-08-27

### Added
- **Import/Module system** — full support for organizing code into modules
  - `use path::to::Item` — import specific items
  - `use path::{A, B, C as D}` — selective imports with aliases
  - `use path::*` — wildcard imports
  - `mod name { ... }` — inline module definitions
  - `mod name;` — file-based module loading (from `name.ruva` or `name/mod.ruva`)
  - Nested modules support (`mod utils { pub mod strings { ... } }`)
- **Enhanced type checker** — now handles use declarations and inline modules
  - Registers imported symbols into scope
  - Validates inline module definitions
  - 9 new tests for import/module parsing
- **Module codegen for all backends**:
  - Rust: generates proper `mod` blocks with items
  - Zig: generates `pub const Module = struct { ... }`
  - Python: generates `class Module:` with methods
- **New example files**:
  - `imports.ruva` — comprehensive import/module demonstration
  - `geometry.ruva` — file-based module example
- **12 new unit tests** (45 total, up from 33)
- **Updated README** with import/module documentation

### Changed
- Enhanced AST with `UseDef`, `UseItem`, and `ModDef.body` for inline modules
- Parser now handles `use` declarations and inline `mod` blocks
- Type checker registers imported symbols and validates module contents
- All three backends generate proper import/module code

## [0.2.0] — 2026-08-27

### Added
- **Multi-target transpilation** — Ruva now transpiles to Rust, Zig, or Python
  - `--target rust` (default) — native binary via cargo
  - `--target zig` — systems language with C interop
  - `--target python` — security-focused, GC-managed, easy to audit
- **Backend abstraction** (`backend.rs`) — `CodeGenerator` trait for pluggable backends
- **Zig codegen** (`codegen_zig.rs`, 1,048 LOC) — structs, enums, methods, control flow
- **Python codegen** (`codegen_python.rs`, 1,041 LOC) — classes, match/case, dataclasses, typing
- **Basic type checker** (`typecheck.rs`) — catches undefined variables, duplicate bindings
  - Integrated into `ruva check` command
  - 5 unit tests for type checking
- **CLI `--target` flag** on `compile`, `transpile`, and `pipe` commands
- **8 new example files** (3,298 LOC total, up from 1,879):
  - `security_focused.ruva` — input validation, rate limiting, audit logging
  - `error_handling.ruva` — Result, Option, try/catch, custom errors
  - `data_structures.ruva` — linked list, BST, graph
  - `patterns.ruva` — match, ranges, or patterns, destructuring
  - `testing.ruva` — pure functions, calculator, fizzbuzz
  - `systems.ruva` — bit ops, ring buffer, memory pool, CRC32, VM
  - `mini_http.ruva` — HTTP server, routing, middleware
  - `async_demo.ruva` — task executor, channels, work queue
- **16 new unit tests** (31 total, up from 10):
  - 5 backend tests (target parsing, display, extensions)
  - 5 Zig codegen tests (hello, struct, function, if/else, while)
  - 6 Python codegen tests (hello, class, function, if/else, while, target)
  - 5 type checker tests (undefined var, duplicate binding, valid code, class, params)

### Changed
- Refactored Rust codegen to implement `CodeGenerator` trait
- Renamed `generate()` to `generate_rust()` to avoid name conflicts
- Updated CLI to use `--target` flag instead of hardcoded Rust output
- `ruva check` now runs both syntax check and type check

### Fixed
- Removed unused `get_dependencies` method from Rust codegen
- Fixed all compiler warnings (0 warnings in release build)

## [0.1.0] — 2026-08-26

### Added
- Initial release of Ruva transpiler
- Lexer with full keyword and operator support
- Pratt parser with operator precedence
- AST types for all language constructs
- Rust codegen with Self, float literals, trait method fixes
- 10 CLI subcommands: compile, build, run, check, transpile, tokens, ast, repl, pipe, new
- 10 unit tests
- 13 example .ruva files
