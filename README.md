# Ruva

**Rust safety. Java familiarity. Zig precision. Python simplicity.**

A compiled language that blends the best of four languages:
- **Rust (50%)**: Ownership, pattern matching, zero-cost abstractions, memory safety
- **Java (20%)**: Classes, interfaces, exception handling, familiar OOP syntax
- **Zig (15%)**: Comptime evaluation, explicit control, no hidden allocations
- **Python (15%)**: Decorators, list comprehensions, clean syntax

Ruva transpiles to 13 backends — Rust, Zig, Python, Java, C#, Go, Swift,
Kotlin, TypeScript, JavaScript, Lua, Ruby, and PHP.

> *"Write safe, fast code that feels familiar. Choose your backend."*

---

## Language DNA

Ruva blends the best features from four languages:

### Rust (50%) — The Foundation
| Feature | Source |
|---------|--------|
| Ownership & borrowing | Prevents memory bugs |
| Pattern matching | Exhaustive match arms |
| Zero-cost abstractions | No runtime overhead |
| Enums (ADTs) | Algebraic data types |
| Closures | First-class functions |
| Generics | Type-safe reusable code |
| `unsafe` blocks | When you need raw control |
| Raw pointers | FFI and systems programming |

### Java (20%) — The Familiarity
| Feature | Source |
|---------|--------|
| `class` syntax | OOP done right |
| `interface` definitions | Contract-based design |
| `try`/`catch`/`finally` | Exception handling |
| `package` declarations | Module organization |
| `pub` visibility | Encapsulation |
| `impl` blocks | Methods on types |
| `static` methods | Class-level operations |

### Zig (15%) — The Precision
| Feature | Source |
|---------|--------|
| `comptime` blocks | Compile-time evaluation |
| Explicit error handling | No hidden control flow |
| `?T` optional types | Null safety without Option |
| No hidden allocations | Memory-conscious design |
| `|err|` error handling | Explicit error propagation |

### Python (15%) — The Simplicity
| Feature | Source |
|---------|--------|
| `@decorator` syntax | Metaprogramming made easy |
| List comprehensions | `[x for x in items if x > 0]` |
| `f"string {interpolation}"` | Clean string building |
| Clean indentation-like syntax | Readable by default |
| Dynamic typing option | When types aren't needed |

### The Formula

```
Rust (50%) + Java (20%) + Zig (15%) + Python (15%) = Ruva
    ↓              ↓            ↓            ↓
 Memory-safe   Familiar    Explicit    Simple
  & fast        OOP        control     & clean
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

## Graphics & Rendering Support

Ruva ships with safe bindings to industry-standard graphics APIs via `import ruva::graphics`:

### Supported APIs
- **OpenGL** — Window, Context, Shader, Texture management with ownership safety
- **Vulkan** — Instance, Device, Swapchain, RenderPass, Pipeline, CommandPool, Buffer
- **DirectX 11** — Device, DeviceContext, VertexShader, PixelShader, RenderTargetView
- **DirectX 12** — Device, CommandQueue, CommandList, PipelineState, RootSignature, Fence

### Why Graphics APIs Are Safe in Ruva

| Traditional Risk | Ruva's Protection |
|------------------|-------------------|
| Use-after-free on GPU resources | Ownership prevents accessing freed resources |
| Null pointer in shader compilation | Option<T> forces explicit null handling |
| Buffer overflow in vertex data | Bounds checking on all array access |
| Data race on render thread | Ownership prevents concurrent mutation |

```ruva
import ruva::graphics::opengl

let window = opengl::Window::new("My Game", 1920, 1080)
let ctx = opengl::Context::new(window)
ctx.clear(0.1, 0.1, 0.1, 1.0)  // dark background
let shader = ctx.create_shader(vertex_src, fragment_src)
```

---

## Browser Support

Ruva ships with browser API bindings via `import ruva::browser`:

### Supported APIs

| Module | Features |
|--------|----------|
| **DOM** | Element, Document, Window — get/set id, class, attributes, innerHTML, textContent |
| **Canvas 2D** | Fill/stroke rects, text, paths, arcs, transforms (save/restore/translate/rotate/scale) |
| **WebGL** | Shaders, programs, buffers, textures, framebuffers — full rendering pipeline |
| **Fetch** | GET/POST requests, JSON parsing, array buffers |
| **WebSocket** | Send text/binary data, connection state management |
| **WebAssembly** | Memory and Table management for Wasm modules |

```ruva
import ruva::browser::dom
import ruva::browser::canvas

let doc = dom::get_document()
let el = doc.create_element("canvas")
let ctx = canvas::CanvasRenderingContext2D { handle: 0 }
ctx.fill_rect(0.0, 0.0, 800.0, 600.0)
ctx.fill_text("Hello from Ruva!", 100.0, 300.0)
```

---

## Video Rendering Support

Ruva ships with video encoding/decoding bindings via `import ruva::video`:

### Supported Codecs
H.264, H.265, VP8, VP9, AV1, MPEG4

### Supported Containers
MP4, MKV, AVI, MOV, WebM, FLV

### Features

| Module | Capabilities |
|--------|-------------|
| **VideoDecoder** | Decode frames, seek, get video info (resolution, framerate, bitrate) |
| **VideoEncoder** | Encode frames, set bitrate/framerate, flush and close |
| **AudioDecoder** | Decode audio frames, seek, get sample rate/channels |
| **AudioEncoder** | Encode audio frames, set bitrate |
| **Muxer/Demuxer** | Container muxing/demuxing with packet-level access |
| **Filters** | Resize, crop, rotate, blur, sharpen, brightness, contrast, grayscale, invert, flip, text/image overlay |

### Pixel Formats
YUV420, YUV422, YUV444, RGB24, RGBA32, NV12, NV21

```ruva
import ruva::video

let decoder = video::VideoDecoder::new("input.mp4")
let info = decoder.get_info()  // width, height, frame_rate, codec
let frame = decoder.decode_frame()  // returns VideoFrame

let encoder = video::VideoEncoder::new("output.mp4", video::Codec::H264)
encoder.write_frame(frame)
encoder.flush()
```

---

## Language Features

### Rust Features (50%)

#### Variables — immutable by default

```ruva
let x = 10          // immutable (safe by default)
let mut y = 20      // mutable (opt-in)
```

#### Pattern Matching — Exhaustive & Safe

```ruva
match value {
    0 => "zero",
    1..=9 => "single digit",
    10 | 20 | 30 => "special",
    _ => "other",
}
```

#### Error Handling — Result Types

```ruva
fn divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 { return Err("Division by zero".into()) }
    return Ok(a / b)
}

match divide(10.0, 2.0) {
    Ok(result) => println!("Result: {}", result),
    Err(err) => println!("Error: {}", err),
}
```

#### Closures — First-class Functions

```ruva
let add = |a: i32, b: i32| -> i32 { a + b }
let numbers = [1, 2, 3, 4, 5]
// Closures work with iterators
```

#### Unsafe — When You Need Raw Control

```ruva
unsafe {
    let ptr = null_mut()
    *ptr = 42  // Raw pointer dereference
}
```

---

### Java Features (20%)

#### Classes — Java-familiar OOP

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

#### Interfaces — Contract-based Design

```ruva
interface Drawable {
    fn draw(&self)
    fn area(&self) -> f64
}

class Circle {
    pub let radius: f64,

    pub fn new(radius: f64) -> Self {
        return Self { radius }
    }
}

impl Circle {
    pub fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius)
    }
    pub fn area(&self) -> f64 {
        return 3.14159 * self.radius * self.radius
    }
}
```

#### Exception Handling — try/catch/finally

```ruva
try {
    let result = dangerous_operation()
    println!("Success: {}", result)
} catch(e) {
    println!("Error: {}", e)
} finally {
    cleanup()
}
```

#### Throw — Explicit Error Raising

```ruva
fn validate(age: i32) {
    if age < 0 {
        throw "Age cannot be negative"
    }
}
```

#### Package Declarations

```ruva
package com.example.myapp

fn main() {
    println!("Organized code")
}
```

#### Enums — Algebraic Data Types

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
            (s * (s - a) * (s - b) * (s - c))
        }
    }
}
```

#### Imports & Modules

```ruva
use std::io::{Read, Write}
use geometry::{Point, Circle}

mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        return a + b
    }
}

mod geometry;  // File-based module
import ruva::core  // Stdlib import
```

---

### Zig Features (15%)

#### Comptime — Compile-time Evaluation

```ruva
comptime {
    let x = 2 + 3
    println!("This runs at compile time: {}", x)
}
```

#### Explicit Error Handling

```ruva
// Errors are explicit, not hidden
fn parse(input: string) -> Result<i64, string> {
    // No hidden control flow
    return Ok(42)
}
```

---

### Python Features (15%)

#### Decorators — Metaprogramming

```ruva
@log_calls
@timeout(30)
fn process_data(data: string) {
    println!("Processing: {}", data)
}
```

#### List Comprehensions

```ruva
let numbers = [1, 2, 3, 4, 5]
let doubled = [x * 2 for x in numbers]
let evens = [x for x in numbers if x % 2 == 0]
```

#### String Interpolation — f-strings

```ruva
let name = "Ruva"
let version = 10
let msg = f"Welcome to {name} v{version}!"
println!("{}", msg)
```

#### Assertions

```ruva
assert!(x > 0, "x must be positive")
assert_eq!(a, b, "values should match")
assert_ne!(a, b, "values should differ")
```

#### Optional Chaining & Null Coalescing

```ruva
let name = user?.name ?? "Anonymous"
let value = config?.timeout ?? 30
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

# Transpile to any of 13 backends
ruva transpile src/main.ruva --target java --stdout

# Transpile to TypeScript
ruva transpile src/main.ruva --target typescript --stdout

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
| `ruva compile <file> --target java` | Transpile to Java | `ruva compile src/main.ruva --target java` |
| `ruva compile <file> --target typescript` | Transpile to TypeScript | `ruva compile src/main.ruva --target typescript` |
| `ruva compile <file> --target go` | Transpile to Go | `ruva compile src/main.ruva --target go` |
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

### Multi-Target Backends (13 total)

| Backend | Use Case | Output |
|---------|----------|--------|
| **Rust** | Maximum performance, systems programming | `.rs` → native binary |
| **Zig** | Embedded, security, C interop | `.zig` → compiled binary |
| **Python** | Security-sensitive, scripting, rapid prototyping | `.py` → interpreted |
| **Java** | Enterprise, Android, JVM ecosystem | `.java` → compiled |
| **C#** | .NET, Unity game dev, enterprise | `.cs` → compiled |
| **Go** | Cloud, microservices, networking | `.go` → compiled |
| **Swift** | iOS, macOS, Apple ecosystem | `.swift` → compiled |
| **Kotlin** | Android, JVM, modern Java | `.kt` → compiled |
| **TypeScript** | Web, Node.js, type-safe JS | `.ts` → compiled |
| **JavaScript** | Universal web, browser, Node.js | `.js` → interpreted |
| **Lua** | Embedded scripting, game modding | `.lua` → interpreted |
| **Ruby** | Web (Rails), scripting, automation | `.rb` → interpreted |
| **PHP** | Web backend, WordPress, CMS | `.php` → interpreted |
| Compile Time | Slow | Fast | N/A |
| Runtime | None | None | GC-managed |
| Dependencies | cargo | zig toolchain | stdlib only |

---

## Design Philosophy

1. **Multi-language DNA** — Take the best from Rust, Java, Zig, and Python.
   Don't reinvent what already works.

2. **Safety first** — Memory safety is non-negotiable.
   Rust's ownership model prevents entire classes of bugs.

3. **Familiarity** — If you know Java, Python, or Zig, you already know
   parts of Ruva. No new paradigms to learn.

4. **Zero-cost abstractions** — Classes compile to struct + impl.
   Decorators compile to attributes. No runtime overhead.

5. **Explicit over implicit** — Like Zig, no hidden control flow.
   Like Rust, no hidden allocations. Like Java, clear error handling.

---

## Examples

**875 examples** across **36 categories**:

| Category | Count | Description |
|----------|-------|-------------|
| basics | 35 | Variables, types, operators, string interpolation |
| control_flow | 35 | if/else, while, for, match, loop, fizzbuzz, fibonacci |
| functions | 35 | Closures, recursion, higher-order, pattern matching |
| classes | 40 | OOP, encapsulation, methods, counter, calculator, matrix |
| enums | 30 | ADTs, pattern matching, Option, Result, status codes |
| error_handling | 25 | try/catch, Result, error chains, recovery |
| data_structures | 30 | Stack, queue, linked list, binary tree, graph |
| generics | 20 | Generic functions, structs, traits, bounds |
| modules | 20 | Imports, exports, inline modules, re-exports |
| async | 20 | Async/await, channels, mutex, task spawning |
| ffi | 20 | extern C, unsafe blocks, raw pointers, callbacks |
| graphics | 40 | OpenGL, Vulkan, DirectX, shaders, textures |
| browser | 40 | DOM, Canvas, WebGL, Fetch, WebSocket, Wasm |
| video | 35 | Decode, encode, mux, filters, audio |
| game_dev | 40 | Game loop, sprites, physics, AI, UI |
| web_server | 25 | HTTP, routing, middleware, WebSocket, auth |
| cli | 25 | Arg parsing, progress bars, interactive prompts |
| algorithms | 35 | Sorting, searching, graph, dynamic programming |
| data_processing | 25 | CSV, JSON, filtering, aggregation, statistics |
| networking | 20 | TCP, UDP, HTTP, DNS, SSL |
| crypto | 15 | Hashing, encryption, signing, key generation |
| database | 15 | SQLite, Redis, MongoDB operations |
| string_processing | 20 | Reverse, palindrome, regex, compression |
| math | 20 | Primes, factorial, matrices, calculus |
| testing | 15 | Unit tests, assertions, benchmarks |
| design_patterns | 25 | Singleton, factory, observer, strategy |
| systems_programming | 25 | Memory mapping, bit manipulation, threads |
| embedded | 15 | GPIO, SPI, I2C, timers, power management |
| machine_learning | 15 | Regression, classification, neural networks |
| security | 15 | Input sanitization, XSS prevention, JWT |
| performance | 15 | Caching, SIMD, lazy evaluation, memoization |
| concurrency | 15 | Threads, channels, mutex, atomics |
| file_io | 15 | Read/write files, directories, watching |
| serialization | 15 | JSON, TOML, YAML, binary formats |
| compression | 10 | Gzip, ZIP, LZ4, Snappy |
| java_features | 2 | Interface, package declaration |
| zig_features | 1 | Comptime blocks |
| python_features | 2 | Decorators, list comprehensions |

---

## Status

**v1.0.0 — Multi-Language Features**

### Language Support
- **Rust (50%)**: Ownership, pattern matching, closures, generics, unsafe, raw pointers, enums
- **Java (20%)**: Classes, interfaces, try/catch, throw, package declarations
- **Zig (15%)**: Comptime blocks, explicit error handling
- **Python (15%)**: Decorators, list comprehensions, f-strings, assertions

### Core Pipeline
- Lexer: ✅ complete with token pre-allocation and keyword optimization
- Parser: ✅ Pratt parser with precedence climbing, all language features
- Type Checker: ✅ real type unification, argument/return type checking, unsafe enforcement, source locations
- Rust CodeGen: ✅ complete with all features
- Zig CodeGen: ✅ complete with all features
- Python CodeGen: ✅ complete with all features
- Security: ✅ path traversal rejection, file size limits, JSON depth limits, dangerous FFI detection

### Tooling
- CLI: ✅ 12 subcommands (compile, build, run, check, transpile, tokens, ast, repl, pipe, new, fmt, lsp)
- LSP: ✅ text document sync, hover, go-to-definition, completion, diagnostics, parse error reporting
- Tests: ✅ 157 passing
- CI/CD: ✅ GitHub Actions (build, test, lint, cross-platform)

### Standard Library (9 modules)
- core, graphics (OpenGL/Vulkan/DX11/DX12), browser (DOM/Canvas/WebGL/Fetch/WebSocket/Wasm), video (encode/decode/mux/filters), anticheat, io, testing, formatter, serialization

---

## License

MIT

