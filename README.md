# Ruva

**Easy to learn. Fast. Secure.**

Ruva is its own language — modern syntax, memory safety, predictable performance, and a clean mental model. It runs directly on its own bytecode VM and compiles to native via its Rust backend.

```
Ruva = The Language
```

> *"Write safe, fast code that feels natural. Run it anywhere you like."*

---

## Quick Start

```bash
# Build from source
cd Ruva
cargo build --release
cargo install --path .

# Create a new project
ruva new my_project
cd my_project

# Run a Ruva file
ruva run src/main.rve

# Compile to native binary (Rust backend)
ruva compile src/main.rve -o my_app

# Transpile to Rust source
rgu build src/main.rve --stdout

# Run through the bytecode VM (no build step)
rgu run src/main.rve

# Check for errors
ruva check src/main.rve

# Start the LSP server
ruva lsp
```

---

## Language Features

### Core language

```ruva
// Immutable by default, opt-in mutability
let x = 10
let mut y = 20

// Pattern matching with exhaustive arms
match value {
    0 => "zero",
    1..=9 => "single digit",
    10 | 20 | 30 => "special",
    _ => "other",
}

// Result types with explicit error handling
fn divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 { return Err("Division by zero".into()) }
    return Ok(a / b)
}

// Closures, generics, unsafe blocks, raw pointers, enums (ADTs)
let add = |a: i32, b: i32| -> i32 { a + b }
struct Stack<T> { items: Vec<T> }
```

### Classes and objects

```ruva
// Classes with encapsulation
class Person {
    pub let name: string,
    pub let mut age: u32,

    pub fn new(name: string, age: u32) -> Self {
        return Self { name, age }
    }

    pub fn birthday(&mut self) {
        self.age += 1
    }
}

// Interfaces, try/catch/finally, throw, package declarations
interface Drawable {
    fn draw(&self)
    fn area(&self) -> f64
}
```

### Compile-time evaluation

```ruva
// Compile-time evaluation
comptime {
    let x = 2 + 3
    println!("This runs at compile time: {}", x)
}

// Explicit error handling, no hidden control flow
```

### Everyday conveniences

```ruva
// Decorators
@log_calls
@timeout(30)
fn process_data(data: string) { }

// List comprehensions
let doubled = [x * 2 for x in numbers]
let evens = [x for x in numbers if x % 2 == 0]

// f-strings, assertions, optional chaining, null coalescing
let msg = f"Welcome to {name} v{version}!"
let name = user?.name ?? "Anonymous"
```

---

## How It Works

Ruva source files use the `.rve` (or `.ruva`) extension.

```
  .rve source
       │
       ▼
   ┌────────┐
   │ Lexer  │  → tokens                    (591 LOC)
   └────┬───┘
        │
        ▼
   ┌────────┐
   │ Parser │  → AST                       (2,754 LOC)
   └────┬───┘   Pratt parser with precedence climbing
        │
        ▼
   ┌──────────┐
   │  Type    │  → typed AST with diagnostics  (1,634 LOC)
   │  Checker │
   └────┬─────┘
        │
        ▼
   ┌──────────┐
   │  Module  │  → resolved AST (stdlib inlined)
   │ Resolver │
   └────┬─────┘
        │
        ▼
   ┌─────────┐
   │ CodeGen │  → Rust source → native binary
   └────┬────┘
        │
        ├──→ .rs (Rust) → rustc → native binary
        └──→ bytecode   → rgu/ruva vm (interpreted)
```

---

## CLI Reference

| Command | Description | Example |
|---------|-------------|---------|
| `ruva new <name>` | Create a new project | `ruva new my_app` |
| `ruva run <file>` | Build with rustc and run | `ruva run src/main.rve` |
| `ruva compile <file>` | Build to native (Rust) | `ruva compile src/main.rve -o app` |
| `ruva compile <file> --release` | Optimized build | `ruva compile src/main.rve --release` |
| `ruva compile <file> --lazy` | Syntax check only | `ruva compile src/main.rve --lazy` |
| `ruva build [dir]` | Build all .rve/.ruva in src/ | `ruva build` |
| `ruva check <file>` | Type-check a file | `ruva check src/main.rve` |
| `ruva check <dir> --all` | Check all files | `ruva check src/ --all` |
| `ruva fmt <file>` | Format a file | `ruva fmt src/main.rve` |
| `ruva fmt <dir>` | Format directory | `ruva fmt src/` |
| `ruva fmt --check` | Check format only | `ruva fmt src/main.rve --check` |
| `ruva repl` | Interactive REPL | `ruva repl` |
| `ruva lsp` | Start LSP server | `ruva lsp` |
| `ruva tokens <file>` | Print token stream | `ruva tokens src/main.rve` |
| `ruva ast <file>` | Print AST | `ruva ast src/main.rve` |
| `rgu run <file>` | Run via the VM (no cargo) | `rgu run src/main.rve` |
| `rgu check <file>` | Parse + resolve only | `rgu check src/main.rve` |
| `rgu build <file> [--stdout]` | Transpile to Rust via driver | `rgu build src/main.rve --stdout` |

---

## Standard Library

13 modules available via `import ruva::<module>`:

| Module | Description |
|--------|-------------|
| `core` | Core types, Option, Result, iterators |
| `kernel` | Bare-metal OS dev: memory, interrupts, drivers, scheduler, syscalls |
| `graphics` | OpenGL, Vulkan, DirectX 11/12 bindings |
| `browser` | DOM, Canvas 2D, WebGL, Fetch, WebSocket, WebAssembly |
| `video` | H.264/H.265/VP8/VP9/AV1 encode/decode, mux/demux, filters |
| `anticheat` | Process memory scanning, integrity verification, tamper detection |
| `io` | File I/O, buffered readers/writers |
| `server` | HTTP server, routing, middleware |
| `game` | Game loop, ECS, physics, sprites |
| `testing` | Unit test helpers, assertions, benchmarks |
| `formatter` | Code formatting utilities |
| `serialization` | JSON, TOML, YAML serialization |
| `interop` | FFI helpers, C interop |

---

## LSP Server

Full Language Server Protocol implementation (3,738 LOC):

- **Text document sync** — incremental updates, open/close/change
- **Hover** — type information, keyword docs
- **Go to definition** — jump to symbol definitions
- **Completion** — context-aware suggestions with trigger characters (`.` `:` `::`)
- **Diagnostics** — real-time error/warning reporting
- **Document symbols** — outline view for functions, structs, classes, enums
- **References** — find all usages across open documents
- **Rename** — rename symbols across all open files
- **Signature help** — function signature hints
- **Code actions** — quick fixes
- **Workspace symbol** — search symbols across workspace

```bash
# Start the LSP server (communicates via stdin/stdout JSON-RPC)
ruva lsp
```

---

## Safety Guarantees

Ruva inherits Rust's safety model through transpilation:

| Guarantee | How |
|-----------|-----|
| No segfaults | Rust's ownership model |
| No null pointers | `Option<T>` type |
| No buffer overflows | Bounds checking |
| No data races | Ownership prevents concurrent mutation |
| No use-after-free | Ownership prevents accessing freed memory |
| No memory leaks | RAII (ownership drop semantics) |

| Crash Type | Cause | Blame |
|------------|-------|-------|
| Compile error | Ruva transpiler bug | Ruva |
| Runtime panic | Bounds check, overflow, unwrap on None | Ruva (expected behavior) |
| Segfault | Impossible (Rust prevents it) | N/A |
| Null pointer | Impossible (`Option<T>`) | N/A |
| Data race | Impossible (ownership) | N/A |
| GC pause | Impossible (no GC) | N/A |

---

## Project Structure

```
Ruva/
├── src/                    # Compiler source (17,849 LOC)
│   ├── main.rs             # CLI entry point (11 subcommands)
│   ├── lib.rs              # Library target for integration tests
│   ├── ast.rs              # Token and AST node definitions
│   ├── lexer.rs            # Byte-level tokenizer (591 LOC)
│   ├── parser.rs           # Pratt parser (2,754 LOC)
│   ├── typecheck.rs        # Type checker with diagnostics (1,634 LOC)
│   ├── module.rs           # Module resolution (stdlib + file-based)
│   ├── backend.rs          # CodeGenerator trait + Target enum
│   ├── codegen.rs          # Rust backend (primary)
│   ├── bin/rgu.rs          # RGu compiler driver
│   ├── lsp.rs              # LSP server (3,738 LOC)
│   ├── vm.rs               # Bytecode VM: compiler + interpreter (~1,420 LOC)
│   ├── json_protocol.rs    # Zero-dependency JSON parser/serializer
│   ├── features.rs         # Security feature flags
│   ├── colors.rs           # ANSI terminal colors
│   └── debug.rs            # Token stream printer
├── tests/                  # Integration tests
│   ├── golden_tests.rs     # 24 golden/snapshot tests
│   ├── vm_tests.rs         # 35 bytecode-VM regression tests
│   ├── transpiler_golden/  # .ruva input files for golden tests
│   └── golden/             # Expected output snapshots
├── examples/               # 6,766 example .ruva files across 63 categories
├── stdlib/                 # 13 standard library modules
│   ├── core/               # Core types and utilities
│   ├── kernel/             # Bare-metal OS development
│   ├── graphics/           # OpenGL, Vulkan, DirectX
│   ├── browser/            # DOM, Canvas, WebGL, Fetch
│   ├── video/              # Encode/decode/mux
│   ├── anticheat/          # Process integrity
│   ├── io/                 # File I/O
│   ├── server/             # HTTP server
│   ├── game/               # Game engine
│   ├── testing/            # Test helpers
│   ├── formatter/          # Code formatting
│   ├── serialization/      # JSON/TOML/YAML
│   └── interop/            # FFI helpers
├── Cargo.toml              # Dependencies: clap + anyhow only
└── DESIGN.md               # Language specification
```

---

## Self-Hosting Progress

The Ruva compiler is progressively being rewritten in Ruva itself. The `self_hosted/` directory contains `.ruva` source files that are transpiled to Rust and used as drop-in replacements for the original modules.

```
self_hosted/
└── src/
    ├── colors.ruva              # ANSI color codes (52 LOC)
    └── features.ruva            # Security feature flags (44 LOC)
```

### Self-hosted modules

| Module | .ruva LOC | .rs LOC | Status |
|--------|-----------|---------|--------|
| `colors.rs` | 52 | 66 | ✅ Replaced |
| `features.rs` | 44 | 117 | ✅ Transpiled |
| **Total** | **96** | **183** | |

### How it works

1. Write the module in `.ruva` (e.g., `self_hosted/src/colors.ruva`)
2. Transpile: `rgu build self_hosted/src/colors.ruva`
3. Post-process: fix type aliases, derive attributes, enum variant syntax
4. Copy to `src/` as a drop-in replacement
5. All existing tests continue to pass

### Limitations

The transpiler currently does not support:
- Top-level `const`/`let` (constants must be defined in Rust)
- `Self::` path syntax in impl blocks
- `return` inside match arms
- Inline-struct enum variants (become tuple variants)
- `string` type maps to Rust's `String` (needs `type string = String;` alias)

As these features are added, more modules can be self-hosted.

---

## Bytecode VM

Ruva ships a real bytecode interpreter (`ruva vm file.rve`) that runs Ruva
**directly — no transpilation step** — making it a genuine compiled-language
interpreter rather than a source-to-source transpiler.

```bash
# Run a file through the bytecode VM
ruva vm src/main.rve

# Disassemble the emitted bytecode (debug)
ruva vm src/main.rve --debug
```

Implemented features:

- **Checked arithmetic** — integer add/sub/mul/div/rem and negation use
  `checked_*`, so overflow, division-by-zero, and negation overflow return a
  clean VM error instead of wrapping or panicking.
- **First-class closures** — anonymous functions capture enclosing locals by
  value into a heap environment (`Rc<RefCell<...>>`). Nested closures share the
  same mutable cell across the chain, so state set by an inner closure is
  visible to the outer one. Zero-parameter closures (`|| { ... }`) and
  `|| -> Type { ... }` forms are supported, with ordinary logical OR `a || b`
  preserved.
- **Loop control** — `break` and `continue` work inside `while`, `for-in`, and
  `loop` bodies, compiled to forward/backward jumps. Breaking out of a loop
  leaves the enclosing loop's captures/condition intact.
- **Arrays** — negative and out-of-range indexing are bounds-safe (negative
  counts from the end; out-of-range yields `nil`).
- **String / array safety** — string-repeat rejects negative counts and guards
  against allocation-size overflow.
- **Resource limits** — the call frame stack is bounded (overflow returns an
  error rather than exhausting the host stack).

```
.rve source
     │
     ▼
 Lexer → Parser → Compiler (direct AST → bytecode) → VM interpreter
```

## RGu — the Ruva compiler driver

RGu (`rgu`) is Ruva's own compiler front-end, built as a standalone binary that
**does not need cargo (or any build tool) at runtime**. It links the Ruva
library and drives a `.rve`/`.ruva` file directly through the bytecode VM.

```bash
# Run a .rve file through the VM — no cargo, no build step
rgu run src/main.rve

# Parse + resolve modules (no execution)
rgu check src/main.rve

# Transpile to Rust source (print to stdout, or write a file)
rgu build src/main.rve --stdout

rgu --version
```

Once the `rgu` binary is built, it is self-contained: it reads source, lexes,
parses, resolves modules, and interprets. This is the driver, distinct from the
`ruva` CLI which also offers check/lsp/format tooling (and whose `run`/`compile`
paths build native binaries directly with rustc).

## Dependencies

Minimal — only 2 external crates:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }  # CLI argument parsing
anyhow = "1"                                      # Error handling
```

Everything else is hand-rolled: the lexer, parser, type checker, the bytecode VM, the Rust codegen backend, the LSP server, and the JSON parser for the LSP wire protocol.

---

## Testing

**284 tests, all passing:**

| Category | Count | Description |
|----------|-------|-------------|
| Unit tests (lib) | 71 | Lexer, parser, type checker, Rust codegen |
| Unit tests (bin) | 154 | CLI, module resolution, JSON, LSP |
| Golden tests | 24 | Snapshot regression tests for transpiler output |
| VM tests | 35 | Bytecode VM: closures, loops, break/continue, arithmetic safety |

### Golden Tests

Snapshot tests that compare transpiler output against baseline `.golden` files:

```bash
# Run golden tests
cargo test --test golden_tests

# Regenerate baselines after intentional changes
GOLDEN_BLESS=1 cargo test --test golden_tests
```


---

## Repository Findings

Non-defect observations from a recent codebase survey (kept out of normal bug
lists because they are gaps or organizational notes, not broken behavior):

- **Source file extensions** — `.rve` is a first-class alias for `.ruva` across
the CLI, module resolution, and test harnesses.
- **Self-hosting is mid-flight** — the repo has both `self_hosted/` (transpiled
drop-in modules) and a separate `self-hosting/` Ruva compiler project. The two
overlap in intent; neither is fully committed.
- **Cargo-free native builds** — `ruva compile`/`run` build native binaries with
`rustc` (no cargo at runtime). Programs that import external crates
(`ruva::graphics`, `video`, `anticheat`) need the bytecode VM (`rgu run`) until
a std-only runtime covers them.
- **GitHub language color** — `.gitattributes` maps `.rve`/`.ruva` to the
`Ruva` linguist language (with `linguist-detectable`). Whether the red/reddish
swatch renders still depends on Ruva being registered in linguist upstream.

---

## Target Domains

Ruva is designed for **safety-critical, performance-sensitive** applications:

### Operating Systems
- Bare-metal / `no_std`-style compilation target
- No GC, no hidden allocations — predictable at the kernel level
- Ownership model applies to raw memory and hardware resources

### Anticheats
- Real-time memory scanning without reflection bypass
- Process integrity verification
- Ownership prevents unauthorized memory access

### Server Hosting
- High-conconcurrency servers without thread exhaustion
- Low latency — no GC pauses between requests
- Memory-safe request handling at native performance

---

## Design Philosophy

1. **One language** — Ruva stands on its own. It isn't a wrapper around another language.
2. **Safety first** — Memory safety and panic-free behavior are non-negotiable.
3. **Easy to learn** — A small, consistent feature set with no hidden control flow.
4. **Zero-cost abstractions** — Classes compile to struct + impl. Decorators compile to attributes. No runtime overhead.
5. **Explicit over implicit** — No hidden control flow, no hidden allocations, clear error handling.
6. **Minimal dependencies** — Only `clap` and `anyhow`. Everything else is hand-rolled.

---

## License

MIT
