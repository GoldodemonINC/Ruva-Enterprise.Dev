# Ruva

**Java security. Rust speed. Zero compromise.**

A compiled language that combines Java's familiar OOP syntax with Rust's
memory safety and native performance. Ruva transpiles to multiple backends
— Rust, Zig, or Python — giving you the right tool for every job.

> *"Write safe, fast code in Java syntax. Choose your backend: Rust for speed,
> Zig for embedded, Python for security."*

---

## The Vision

| What Ruva Takes from Java | What Ruva Takes from Rust |
|--------------------------|---------------------------|
| `class` syntax | Memory safety (no segfaults) |
| OOP patterns | Ownership model (no data races) |
| Familiar keywords | Zero-cost abstractions (no overhead) |
| Encapsulation | No garbage collector (no GC pauses) |
| Method syntax | Pattern matching (no switch fallthrough) |

**The formula:**

```
Java's familiar OOP  +  Rust's safety & speed  =  Ruva
         ↓                        ↓
    Easy to learn          Hard to break
    Easy to read           Fast to run
    Easy to maintain       Safe by default
```

---

## Why Ruva?

### The Problem with Java

| Issue | Impact |
|-------|--------|
| **GC pauses** | Frame drops in games, latency spikes in servers |
| **NullPointerException** | #1 cause of Java crashes |
| **Slow startup** | 1-5 seconds JVM warmup |
| **High memory** | Object headers, GC overhead |
| **No memory safety** | Buffer overflows via JNI |

### The Problem with Rust

| Issue | Impact |
|-------|--------|
| **Steep learning curve** | Lifetimes, borrow checker, ownership syntax |
| **Complex syntax** | `&'a mut dyn Trait` is intimidating |
| **Slow iteration** | Compiler fights you on valid code |
| **Fewer developers** | Harder to hire Rust devs |

### Ruva's Solution

| Benefit | How |
|---------|-----|
| **Java familiarity** | `class`, `pub`, `new()` — Java devs productive in hours |
| **Rust performance** | Transpiles to native code — no GC, no JIT |
| **Rust safety** | Ownership model — no null, no data races, no leaks |
| **Fast development** | Simpler syntax than Rust, faster than Java |

---

## Target Domains

Ruva is designed for **safety-critical, performance-sensitive** applications:

### Browser Engines
- Memory-safe rendering engine without GC pauses
- Tab isolation via ownership model
- No buffer overflows in HTML/CSS parsing
- Native speed for JavaScript execution

### Game Engines
- 60fps game logic without frame drops
- ECS (Entity Component System) support
- Real-time physics without GC hitches
- Memory-safe multiplayer networking

### Anticheats
- Real-time memory scanning without reflection bypass
- Process integrity verification
- Ownership prevents unauthorized memory access
- No undefined behavior — predictable detection

### Server Hosting
- High-concurrency servers without thread exhaustion
- Low latency — no GC pauses between requests
- Memory-safe request handling
- Native performance for thousands of connections

---

## Graphics & Rendering Support (Roadmap)

> ⚠️ **Not yet implemented.** These are planned features.

Ruva will provide safe bindings to industry-standard graphics APIs:

### Planned APIs
- **OpenGL** — Safe bindings with ownership preventing use-after-free
- **Vulkan** — Memory-safe GPU resource management
- **DirectX 11/12** — Safe COM reference counting

### Why Graphics APIs Will Be Safe in Ruva

| Traditional Risk | Ruva's Protection |
|------------------|-------------------|
| Use-after-free on GPU resources | Ownership prevents accessing freed resources |
| Null pointer in shader compilation | Option<T> forces explicit null handling |
| Buffer overflow in vertex data | Bounds checking on all array access |
| Data race on render thread | Ownership prevents concurrent mutation |

---

## Browser Support (Roadmap)

> ⚠️ **Not yet implemented.** These are planned features.

Ruva will compile to WebAssembly for browser deployment:

```bash
# Planned: Compile to Wasm
ruva compile src/main.ruva --target wasm32

# Planned: Compile to JavaScript
ruva compile src/main.ruva --target js
```

### Planned Browser Features

| Feature | Status |
|---------|--------|
| **WebGL rendering** | 🔜 Via OpenGL bindings |
| **WebGPU rendering** | 🔜 Via Vulkan bindings |
| **DOM manipulation** | 🔜 Via web-sys bindings |
| **Service workers** | 🔜 Via wasm-bindgen |
| **Web Workers** | 🔜 Via wasm-bindgen |
| **Fetch API** | 🔜 Via web-sys |
| **Canvas 2D** | 🔜 Via web-sys |
| **Audio** | 🔜 Via web-sys |

---

## Video Rendering Support (Roadmap)

> ⚠️ **Not yet implemented.** These are planned features.

Ruva will provide safe video encoding/decoding:

### Planned Video Features

| Feature | Status |
|---------|--------|
| **H.264 encoding** | 🔜 Via Rust backends |
| **H.265 encoding** | 🔜 Via Rust backends |
| **VP9 encoding** | 🔜 Via Rust backends |
| **AV1 encoding** | 🔜 Via Rust backends |
| **Frame extraction** | 🔜 Via Rust backends |
| **Hardware acceleration** | 🔜 Automatic via Rust backends |

---

## Language Features

### Variables — immutable by default

```ruva
let x = 10          // immutable (safe by default)
let mut y = 20      // mutable (opt-in)
```

### Classes — Java-familiar OOP

```ruva
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
```

### Enums — Algebraic Data Types

```ruva
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64, f64),
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(r) => 3.14159 * r * r,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(a, b, c) => {
            let s = (a + b + c) / 2.0
            (s * (s - a) * (s - b) * (s - c)).sqrt()
        }
    }
}
```

### Error Handling — No Exceptions

```ruva
fn divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 { return Err("Division by zero".into()) }
    return Ok(a / b)
}

// Pattern matching on Results
match divide(10.0, 2.0) {
    Ok(result) => println!("Result: {}", result),
    Err(err) => println!("Error: {}", err),
}
```

### Pattern Matching — Exhaustive & Safe

```ruva
match value {
    0 => "zero",
    1..=9 => "single digit",
    10 | 20 | 30 => "special",
    _ => "other",
}
```

### Imports & Modules — Organized Code

```ruva
// Use declarations
use std::io::{Read, Write}
use math::add
use geometry::{Point, Circle}
use utils::strings as str_utils

// Inline module
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        return a + b
    }
}

// File-based module (loads from geometry.ruva)
mod geometry;

// Legacy import syntax (for stdlib)
import ruva::core
```

---

## Crash Attribution: Ruva vs Rust vs Java

**Question:** "If it crashes, how do you know if it's Ruva's, Rust's, or Java's fault?"

### Answer: Ruva Crashes are Almost Always Ruva's Fault

| Crash Type | Cause | Blame |
|------------|-------|-------|
| **Compile error** | Ruva transpiler bug | Ruva |
| **Runtime panic** | Bounds check, overflow, unwrap on None | Ruva (expected behavior) |
| **Segfault** | Impossible in Ruva (Rust prevents it) | N/A |
| **Null pointer** | Impossible in Ruva (Option<T>) | N/A |
| **Data race** | Impossible in Ruva (ownership) | N/A |
| **GC pause** | Impossible in Ruva (no GC) | N/A |

### Why Ruva Crashes are Ruva's Fault

1. **Ruva generates Rust code** — if the generated code is wrong, it's a Ruva transpiler bug
2. **Rust catches most errors at compile time** — runtime crashes are rare
3. **Ruva doesn't use Java** — Java faults are impossible
4. **Rust's compiler is battle-tested** — Rust bugs are extremely rare

### Crash Debugging

```bash
# See the generated Rust code
ruva transpile src/main.ruva --stdout

# Check for syntax errors
ruva compile src/main.ruva --lazy

# Compile with debug info
ruva compile src/main.ruva -o app

# Run with backtrace
RUST_BACKTRACE=1 ./app
```

### The Safety Guarantees

| Guarantee | How Ruva Enforces It |
|-----------|---------------------|
| **No segfaults** | Rust's ownership model |
| **No null pointers** | Option<T> type |
| **No buffer overflows** | Bounds checking |
| **No data races** | Ownership prevents concurrent mutation |
| **No use-after-free** | Ownership prevents accessing freed memory |
| **No memory leaks** | RAII (ownership drop semantics) |

---

## Quick Start

```bash
# Install (build from source)
cd Ruva
cargo build --release
cargo install --path .

# Create a new project
ruva new my_project
cd my_project

# Run a Ruva file directly
ruva run src/main.ruva

# Compile to native executable (Rust backend)
ruva compile src/main.ruva -o my_app

# Transpile to Zig
ruva transpile src/main.ruva --target zig --stdout

# Transpile to Python (great for security-sensitive code)
ruva transpile src/main.ruva --target python --stdout

# Check for syntax errors (fast, no codegen)
ruva compile src/main.ruva --lazy
```

---

## CLI Reference

| Command | Description | Example |
|---------|-------------|---------|
| `ruva new <name>` | Create a new project | `ruva new my_app` |
| `ruva run <file>` | Compile and run | `ruva run src/main.ruva` |
| `ruva compile <file>` | Build to native (Rust) | `ruva compile src/main.ruva -o app` |
| `ruva compile <file> --target zig` | Build via Zig | `ruva compile src/main.ruva --target zig` |
| `ruva compile <file> --target python` | Transpile to Python | `ruva compile src/main.ruva --target python` |
| `ruva compile <file> --release` | Optimized build | `ruva compile src/main.ruva --release` |
| `ruva compile <file> --lazy` | Syntax check only | `ruva compile src/main.ruva --lazy` |
| `ruva transpile <file>` | Generate target code | `ruva transpile src/main.ruva --stdout` |
| `ruva transpile <file> --target zig` | Generate Zig code | `ruva transpile src/main.ruva --target zig` |
| `ruva transpile <file> --target python` | Generate Python code | `ruva transpile src/main.ruva --target python` |
| `ruva check <file>` | Check syntax | `ruva check src/main.ruva` |
| `ruva check <dir> --all` | Check all files | `ruva check src/ --all` |
| `ruva fmt <file>` | Format a file | `ruva fmt src/main.ruva` |
| `ruva fmt <dir>` | Format directory | `ruva fmt src/` |
| `ruva fmt --check` | Check format only | `ruva fmt src/main.ruva --check` |
| `ruva fmt --dry-run` | Dry run | `ruva fmt src/ --dry-run` |
| `ruva repl` | Interactive REPL | `ruva repl` |
| `ruva lsp` | Start LSP server | `ruva lsp` |

---

## How It Works

```
  .ruva source
       │
       ▼
   ┌────────┐
   │ Lexer  │  → tokens
   └────┬───┘
        │
        ▼
   ┌────────┐
   │ Parser │  → AST
   └────┬───┘
        │
        ▼
   ┌─────────┐
   │ CodeGen │  → target source code
   └────┬────┘
        │
        ├──→ .rs   (Rust backend)   → rustc → native binary
        ├──→ .zig  (Zig backend)    → zig build-exe
        └──→ .py   (Python backend) → python3 (interpreted)
```

### Multi-Target Backends

| Backend | Use Case | Output |
|---------|----------|--------|
| **Rust** | Maximum performance, systems programming | `.rs` → native binary |
| **Zig** | Embedded, security, C interop | `.zig` → compiled binary |
| **Python** | Security-sensitive, scripting, rapid prototyping | `.py` → interpreted |

### Why Multiple Backends?

- **Rust**: When you need raw speed and memory safety. Games, servers, OS.
- **Zig**: When you need embedded-friendly code, manual memory control, or C interop.
- **Python**: When security is paramount. No memory corruption, no buffer overflows, easy to audit.

### Backend Comparison

| Feature | Rust | Zig | Python |
|---------|------|-----|--------|
| Performance | ⭐⭐⭐ | ⭐⭐⭐ | ⭐ |
| Memory Safety | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Security | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Ease of Audit | ⭐ | ⭐⭐ | ⭐⭐⭐ |
| Compile Time | Slow | Fast | N/A |
| Runtime | None | None | GC-managed |
| Dependencies | cargo | zig toolchain | stdlib only |

---

## Design Philosophy

1. **Safety first, performance second** — Memory safety is non-negotiable.
   Performance is a bonus.

2. **Java familiarity** — If you know Java, you already know 80% of Ruva.
   No new paradigms to learn.

3. **Rust power** — Ownership, pattern matching, zero-cost abstractions.
   The hard stuff, made easy.

4. **Classes are sugar** — `class` compiles to the same Rust struct + impl
   blocks you'd write by hand. Zero overhead.

5. **Security through defaults** — No null pointers, no data races, no
   buffer overflows. The type system prevents entire classes of bugs.

---

## Status

**v0.9.0 — LSP (Language Server Protocol)**

- Lexer: ✅ complete
- Parser: ✅ core syntax + if let, as casts, closures, use/mod, generic enums
- Rust CodeGen: ✅ complete with Self, floats, traits, imports, modules
- Zig CodeGen: ✅ structs, enums, methods, control flow, modules
- Python CodeGen: ✅ classes, match/case, dataclasses, typing, modules
- CLI: ✅ 12 subcommands (compile, build, run, check, transpile, tokens, ast, repl, pipe, new, fmt, lsp)
- Tests: ✅ 110 passing (lexer, parser, backends, type checker, module resolver, LSP)
- Examples: ✅ 29 .ruva files (5,000+ LOC)
- Type checker: ✅ variable checking, function args, return types, modules
- Import/Module system: ✅ use declarations, inline modules, file modules
- Standard library: ✅ core, graphics, browser, video, anticheat, io, testing, formatter, serialization
- LSP / editor support: ✅ text document sync, hover, go-to-definition, completion, diagnostics
- Browser/Wasm target: 🔜 planned

---

## License

MIT

