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

**v0.10.0 — Type System & Security Hardening**

- Lexer: ✅ complete with token pre-allocation
- Parser: ✅ core syntax + if let, as casts, closures, use/mod, generic enums, extern blocks, raw pointers
- Rust CodeGen: ✅ complete with Self, floats, traits, imports, modules
- Zig CodeGen: ✅ structs, enums, methods, control flow, modules
- Python CodeGen: ✅ classes, match/case, dataclasses, typing, modules
- CLI: ✅ 12 subcommands (compile, build, run, check, transpile, tokens, ast, repl, pipe, new, fmt, lsp)
- Tests: ✅ 157 passing (lexer, parser, backends, type checker, module resolver, LSP)
- Examples: ✅ 30 .ruva files (5,000+ LOC)
- Type checker: ✅ real type unification, argument/return type checking, unsafe enforcement, source locations
- Security: ✅ path traversal rejection, file size limits, JSON depth limits, dangerous FFI detection
- Import/Module system: ✅ use declarations, inline modules, file modules, path validation
- Standard library: ✅ core, graphics (OpenGL/Vulkan/DX11/DX12), browser (DOM/Canvas/WebGL/Fetch/WebSocket/Wasm), video (encode/decode/mux/filters), anticheat, io, testing, formatter, serialization
- LSP / editor support: ✅ text document sync, hover, go-to-definition, completion, diagnostics, parse error reporting
- Browser/Wasm target: 🔜 planned

---

## License

MIT

