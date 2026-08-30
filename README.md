# Ruva

**Rust safety. Java familiarity. Zig precision. Python simplicity.**

A compiled language that blends the best of four languages and transpiles to 13 backends.

```
Rust (50%) + Java (20%) + Zig (15%) + Python (15%) = Ruva
   ↓              ↓            ↓            ↓
Memory-safe   Familiar    Explicit    Simple
 & fast        OOP        control     & clean
```

> *"Write safe, fast code that feels familiar. Choose your backend."*

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
ruva run src/main.ruva

# Compile to native binary (Rust backend)
ruva compile src/main.ruva -o my_app

# Transpile to any backend
ruva transpile src/main.ruva --target python --stdout
ruva transpile src/main.ruva --target typescript --stdout

# Check for errors
ruva check src/main.ruva

# Start the LSP server
ruva lsp
```

---

## Language Features

### From Rust (50%)

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

### From Java (20%)

```ruva
// Java-familiar classes with encapsulation
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

### From Zig (15%)

```ruva
// Compile-time evaluation
comptime {
    let x = 2 + 3
    println!("This runs at compile time: {}", x)
}

// Explicit error handling, no hidden control flow
```

### From Python (15%)

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

```
  .ruva source
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
   │ CodeGen │  → target source code       (5,748 LOC across 13 backends)
   └────┬────┘
        │
        ├──→ .rs   (Rust)        → cargo build → native binary
        ├──→ .zig  (Zig)         → zig build-exe
        ├──→ .py   (Python)      → python3 (interpreted)
        ├──→ .java (Java)        → javac → JVM bytecode
        ├──→ .cs   (C#)          → dotnet build → .NET
        ├──→ .go   (Go)          → go build → native binary
        ├──→ .swift (Swift)      → swiftc → native binary
        ├──→ .kt   (Kotlin)      → kotlinc → JVM bytecode
        ├──→ .ts   (TypeScript)  → tsc → JS
        ├──→ .js   (JavaScript)  → node (interpreted)
        ├──→ .lua  (Lua)         → lua (interpreted)
        ├──→ .rb   (Ruby)        → ruby (interpreted)
        └──→ .php  (PHP)         → php (interpreted)
```

---

## CLI Reference

| Command | Description | Example |
|---------|-------------|---------|
| `ruva new <name>` | Create a new project | `ruva new my_app` |
| `ruva run <file>` | Compile and run | `ruva run src/main.ruva` |
| `ruva compile <file>` | Build to native (Rust) | `ruva compile src/main.ruva -o app` |
| `ruva compile <file> --target <backend>` | Build via any backend | `ruva compile src/main.ruva --target go` |
| `ruva compile <file> --release` | Optimized build | `ruva compile src/main.ruva --release` |
| `ruva compile <file> --lazy` | Syntax check only | `ruva compile src/main.ruva --lazy` |
| `ruva build [dir]` | Build all .ruva in src/ | `ruva build` |
| `ruva transpile <file>` | Generate target code | `ruva transpile src/main.ruva --stdout` |
| `ruva transpile <file> --target <backend>` | Generate for any backend | `ruva transpile src/main.ruva --target typescript` |
| `ruva check <file>` | Type-check a file | `ruva check src/main.ruva` |
| `ruva check <dir> --all` | Check all files | `ruva check src/ --all` |
| `ruva fmt <file>` | Format a file | `ruva fmt src/main.ruva` |
| `ruva fmt <dir>` | Format directory | `ruva fmt src/` |
| `ruva fmt --check` | Check format only | `ruva fmt src/main.ruva --check` |
| `ruva repl` | Interactive REPL | `ruva repl` |
| `ruva lsp` | Start LSP server | `ruva lsp` |
| `ruva tokens <file>` | Print token stream | `ruva tokens src/main.ruva` |
| `ruva ast <file>` | Print AST | `ruva ast src/main.ruva` |
| `ruva pipe` | Transpile from stdin | `cat file.ruva \| ruva pipe --target rust` |

Valid `--target` values: `rust`, `zig`, `python`, `java`, `csharp`, `go`, `swift`, `kotlin`, `typescript`, `javascript`, `lua`, `ruby`, `php`

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
│   ├── main.rs             # CLI entry point (12 subcommands)
│   ├── lib.rs              # Library target for integration tests
│   ├── ast.rs              # Token and AST node definitions
│   ├── lexer.rs            # Byte-level tokenizer (591 LOC)
│   ├── parser.rs           # Pratt parser (2,754 LOC)
│   ├── typecheck.rs        # Type checker with diagnostics (1,634 LOC)
│   ├── module.rs           # Module resolution (stdlib + file-based)
│   ├── backend.rs          # CodeGenerator trait + Target enum
│   ├── codegen.rs          # Rust backend (primary)
│   ├── codegen_*.rs        # 12 other backends
│   ├── lsp.rs              # LSP server (3,738 LOC)
│   ├── json_protocol.rs    # Zero-dependency JSON parser/serializer
│   ├── features.rs         # Security feature flags
│   ├── colors.rs           # ANSI terminal colors
│   └── debug.rs            # Token stream printer
├── tests/                  # Integration tests
│   ├── golden_tests.rs     # 24 golden/snapshot tests
│   ├── transpiler_bench.rs # Benchmark suite with memory profiling
│   ├── transpiler_golden/  # .ruva input files for golden tests
│   └── golden/             # Expected output snapshots
├── benches/                # Benchmark inputs and runner
│   ├── inputs/             # small.ruva (61 LOC), medium.ruva (232 LOC), large.ruva (722 LOC)
│   └── run.sh              # Benchmark runner script
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
├── benchmarks/             # Multi-language CPU benchmarks
├── Cargo.toml              # Dependencies: clap + anyhow only
└── DESIGN.md               # Language specification
```

---

## Dependencies

Minimal — only 2 external crates:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }  # CLI argument parsing
anyhow = "1"                                      # Error handling
```

Everything else is hand-rolled: the lexer, parser, type checker, all 13 backends, the LSP server, and the JSON parser for the LSP wire protocol.

---

## Testing

**275 tests, all passing:**

| Category | Count | Description |
|----------|-------|-------------|
| Unit tests (lib) | 82 | Lexer, parser, type checker, codegen backends |
| Unit tests (bin) | 165 | CLI, module resolution, JSON, LSP |
| Golden tests | 24 | Snapshot regression tests for transpiler output |
| Benchmarks | 4 | Timing + memory profiling |

### Golden Tests

Snapshot tests that compare transpiler output against baseline `.golden` files:

```bash
# Run golden tests
cargo test --test golden_tests

# Regenerate baselines after intentional changes
GOLDEN_BLESS=1 cargo test --test golden_tests
```

### Benchmarks

Timing and memory profiling across three input sizes:

```bash
# Run all benchmarks
cargo test --release --test transpiler_bench -- --nocapture

# Or use the runner script
bash benches/run.sh
```

Sample output (release mode):

```
  small  (61 LOC)     27µs lex   38µs parse   29µs typecheck   20µs codegen  → 114µs total
                        heap: 30KB lex  56KB parse  6KB typecheck  6KB codegen  → peak 112KB

  medium (232 LOC)    86µs lex  134µs parse   79µs typecheck   54µs codegen  → 353µs total
                        heap: 59KB lex 158KB parse 12KB typecheck 10KB codegen  → peak 262KB

  large  (722 LOC)   235µs lex  420µs parse  260µs typecheck  168µs codegen  → 1.08ms total
                        heap: 205KB lex 486KB parse 45KB typecheck 18KB codegen → peak 795KB
```

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

1. **Multi-language DNA** — Take the best from Rust, Java, Zig, and Python. Don't reinvent what already works.
2. **Safety first** — Memory safety is non-negotiable. Rust's ownership model prevents entire classes of bugs.
3. **Familiarity** — If you know Java, Python, or Zig, you already know parts of Ruva.
4. **Zero-cost abstractions** — Classes compile to struct + impl. Decorators compile to attributes. No runtime overhead.
5. **Explicit over implicit** — No hidden control flow, no hidden allocations, clear error handling.
6. **Minimal dependencies** — Only `clap` and `anyhow`. Everything else is hand-rolled.

---

## License

MIT
