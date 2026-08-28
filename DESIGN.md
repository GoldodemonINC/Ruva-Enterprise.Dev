# Ruva Language Specification v0.1

> **Java structure. Rust power. Zero-cost safety.**

Ruva is a compiled systems language that merges Java's class-oriented design
with Rust's ownership model and performance. It transpiles to Rust, then
compiles to native code via `rustc`/`cargo`.

---

## 1. Philosophy

| Goal | How |
|------|-----|
| **Familiarity** | `class`, `fn`, `let`, `pub` — words you already know |
| **Memory Safety** | Immutable by default, ownership/borrowing at compile time |
| **Performance** | Transpiles to Rust → zero-cost abstractions, no GC |
| **Readability** | Optional semicolons, clean block syntax, no ceremony |
| **Expressiveness** | Pattern matching, algebraic types, traits, error propagation |

---

## 2. Hello World

```ruva
fn main() {
    let message = "Hello, Ruva!"
    println!("{}", message)
}
```

---

## 3. Variables & Mutability

```ruva
let x = 10          // immutable (like Rust's `let`)
let mut y = 20      // mutable
y = 30              // OK — y is `mut`
// x = 40           // ERROR — x is immutable

let name: string = "Buffy"   // explicit type annotation
let age: u32 = 25             // type annotation
```

### Binding Rules
- `let` creates an **immutable** binding (default, safe)
- `let mut` creates a **mutable** binding (opt-in)
- Shadowing is allowed: `let x = 1; let x = x + 1;`
- Every binding has a **type** — inferred or explicit

---

## 4. Primitive Types

| Category | Types |
|----------|-------|
| Signed integers | `i8`, `i16`, `i32`, `i64`, `i128`, `isize` |
| Unsigned integers | `u8`, `u16`, `u32`, `u64`, `u128`, `usize` |
| Floats | `f32`, `f64` |
| Boolean | `bool` (`true` / `false`) |
| Character | `char` (Unicode scalar) |
| String | `string` (UTF-8 owned), `&str` (borrowed slice) |
| Unit | `()` (empty tuple, implicit return) |
| Never | `!` (diverging — panics, infinite loops) |

---

## 5. Compound Types

### 5.1 Arrays (Fixed-size)

```ruva
let nums: [i32; 5] = [1, 2, 3, 4, 5]
let first = nums[0]     // bounds-checked at runtime
```

### 5.2 Slices

```ruva
let slice: &[i32] = &nums[1..4]   // [2, 3, 4]
```

### 5.3 Tuples

```ruva
let pair: (string, u32) = ("Ruva", 1)
let (lang, ver) = pair              // destructuring
```

### 5.4 Option & Result (built-in)

```ruva
let maybe: Option<i32> = Some(42)
let nothing: Option<i32> = None

let ok: Result<u32, string> = Ok(200)
let err: Result<u32, string> = Err("not found".into())
```

---

## 6. Structs

```ruva
struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        return Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        return (dx * dx + dy * dy).sqrt()
    }
}

// Usage
let p = Point::new(3.0, 4.0)
println!("distance = {}", p.distance_to(&Point::new(0.0, 0.0)))
```

### Field Visibility
- Fields are **private** by default (encapsulation like Java)
- `pub` makes a field publicly accessible
- Methods follow the same `pub` / private pattern

---

## 7. Enums (Algebraic Data Types)

```ruva
enum Direction {
    North,
    South,
    East,
    West,
}

enum Shape {
    Circle(f64),                         // tuple variant
    Rectangle { width: f64, height: f64 }, // struct variant
    Triangle(f64, f64, f64),
}

// Usage with pattern matching
fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(r) => {
            return 3.14159 * r * r
        }
        Shape::Rectangle { width, height } => {
            return width * height
        }
        Shape::Triangle(a, b, c) => {
            let s = (a + b + c) / 2.0
            return (s * (s - a) * (s - b) * (s - c)).sqrt()
        }
    }
}
```

---

## 8. Classes

Ruva classes are syntactic sugar over structs + impl blocks.
They provide a Java-familiar `class` keyword that generates
the equivalent Rust struct and implementation.

```ruva
class Person {
    pub let name: string        // immutable field
    pub let mut age: u32        // mutable field

    pub fn new(name: string, age: u32) -> Self {
        return Self { name, age }
    }

    pub fn birthday(&mut self) {
        self.age += 1
    }

    pub fn greet(&self) -> string {
        return format!("Hi, I'm {} and I'm {} years old.", self.name, self.age)
    }
}

// Usage
let mut person = Person::new("Alice".into(), 30)
person.birthday()
println!("{}", person.greet())
```

### `class` vs `struct`
- `struct` = explicit, Rust-native feel
- `class` = Java-familiar, same semantics under the hood
- Use whichever reads better in context — they compile identically

---

## 9. Traits (Interfaces)

```ruva
trait Drawable {
    fn draw(&self)
    fn bounds(&self) -> (f64, f64, f64, f64)   // required
}

trait Area {
    fn area(&self) -> f64 { return 0.0 }       // provided default
}

// Implementing a trait
impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle r={}", self.radius)
    }

    fn bounds(&self) -> (f64, f64, f64, f64) {
        return (self.center.x - self.radius, self.center.y - self.radius,
                self.center.x + self.radius, self.center.y + self.radius)
    }
}

// Trait bounds on generics
fn draw_all(items: &[impl Drawable]) {
    for item in items {
        item.draw()
    }
}
```

---

## 10. Generics

```ruva
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        return Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        return self.items.pop()
    }
}

// Trait bounds
fn max<T: Ord>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}
```

---

## 11. Control Flow

### If / Else
```ruva
if score >= 90 {
    println!("A")
} else if score >= 80 {
    println!("B")
} else {
    println!("C")
}

// If as expression
let grade = if score >= 90 { "A" } else { "B" }
```

### Loops
```ruva
// Infinite loop (has a value via break)
let mut i = 0
let result = loop {
    i += 1
    if i > 100 { break i * 2 }
}

// While loop
while i > 0 {
    i -= 1
}

// For loop (range)
for i in 0..10 {
    println!("{}", i)
}

// For loop (iteration)
let nums = [1, 2, 3, 4, 5]
for n in nums {
    println!("{}", n)
}
```

### Pattern Matching
```ruva
match value {
    0 => println!("zero"),
    1..=9 => println!("single digit"),
    10 | 20 | 30 => println!("special"),
    _ => println!("other"),
}
```

---

## 12. Functions

```ruva
// Basic function
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// Public function
pub fn api_endpoint() -> string {
    return "/v1/users".into()
}

// Early return
fn divide(a: f64, b: f64) -> Result<f64, string> {
    if b == 0.0 {
        return Err("division by zero".into())
    }
    return Ok(a / b)
}

// Closures
let multiply = |a: i32, b: i32| -> i32 { return a * b }
let square = |x: i32| x * x    // implicit return, single expression
```

---

## 13. Error Handling

Ruva uses Rust's `Result<T, E>` with ergonomic sugar.

```ruva
// Explicit Result
fn read_file(path: &str) -> Result<string, IoError> {
    let content = fs::read_to_string(path)?    // ? propagates errors
    return Ok(content)
}

// Try/catch sugar (compiles to match on Result)
fn parse_config(path: string) -> Config {
    try {
        let raw = fs::read_to_string(path)?
        let config: Config = toml::from_str(&raw)?
        return config
    } catch (err) {
        println!("Failed to load config: {}", err)
        return Config::default()
    }
}
```

---

## 14. Ownership & Borrowing

Ruva inherits Rust's ownership model through transpilation.

```ruva
fn consume(s: string) { }         // takes ownership
fn borrow(s: &string) { }         // immutable borrow
fn borrow_mut(s: &mut string) { } // mutable borrow

let name = "Ruva".into()
borrow(&name)          // OK — name is still valid
// consume(name)        // moves name — can't use after
// borrow(&name)        // ERROR — name was moved
```

### Rules (enforced at compile time via Rust backend)
1. Each value has exactly **one owner**
2. When the owner goes out of scope, the value is **dropped**
3. You can have **many immutable borrows** OR **one mutable borrow**
4. Borrows cannot outlive the value they reference

---

## 15. Imports & Modules

```ruva
// Import specific items
import std.io::{println, read_to_string}
import std.collections::HashMap

// Import with alias
import std.net::TcpStream as Stream

// Module declarations (maps to file structure)
// src/main.ruva     → mod main
// src/utils.ruva    → mod utils
// src/utils/mod.ruva → mod utils (with submodules)
```

---

## 16. Strings

```ruva
// String literals (&str — borrowed, stack)
let literal = "hello"

// Owned String (heap-allocated, growable)
let owned = "hello".into()          // string (owned)
let owned2 = String::from("hello")  // explicit constructor

// Formatting
let msg = format!("Hello, {}! You are {}.", name, age)

// String methods (via transpiled Rust)
let upper = text.to_uppercase()
let words = text.split(' ').collect::<Vec<&str>>()
```

---

## 17. Concurrency

```ruva
import std::thread
import std::sync::{Arc, Mutex}

let counter = Arc::new(Mutex::new(0u32))

let mut handles = vec![]
for _ in 0..10 {
    let c = Arc::clone(&counter)
    let handle = thread::spawn(move || {
        let mut num = c.lock().unwrap()
        *num += 1
    })
    handles.push(handle)
}

for h in handles { h.join().unwrap() }
println!("Final count: {}", *counter.lock().unwrap())
```

---

## 18. Attributes & Annotations

```ruva
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[test]
fn test_point_distance() {
    let a = Point::new(0.0, 0.0)
    let b = Point::new(3.0, 4.0)
    assert_eq!(a.distance_to(&b), 5.0)
}
```

---

## 19. Transpilation Pipeline

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
   │ Parser │  → AST (Abstract Syntax Tree)
   └────┬───┘
        │
        ▼
   ┌─────────┐
   │ Type    │  → typed AST (with inferred types resolved)
   │ Checker │
   └────┬────┘
        │
        ▼
   ┌─────────┐
   │ CodeGen │  → Rust source code (.rs)
   └────┬────┘
        │
        ▼
   ┌─────────┐
   │ rustc   │  → native binary
   └─────────┘
```

---

## 20. Keyword Reference

| Keyword | Purpose |
|---------|---------|
| `fn` | Function declaration |
| `let` | Immutable binding |
| `mut` | Mutable modifier |
| `pub` | Public visibility |
| `struct` | Struct declaration |
| `class` | Class declaration (struct + impl sugar) |
| `impl` | Implementation block |
| `trait` | Trait (interface) declaration |
| `enum` | Enum / algebraic type |
| `type` | Type alias |
| `if` / `else` | Conditional |
| `for` | Iterator loop |
| `while` | Condition loop |
| `loop` | Infinite loop |
| `break` | Exit loop (optionally with value) |
| `continue` | Skip to next iteration |
| `return` | Early return |
| `match` | Pattern matching |
| `self` | Instance reference |
| `Self` | Self-type in impl blocks |
| `true` / `false` | Boolean literals |
| `null` | Null reference (maps to Option::None) |
| `use` | Import alias |
| `as` | Type cast / import alias |
| `move` | Ownership transfer into closure |
| `where` | Trait bound clause |
| `derive` | Auto-derive attributes |
| `test` | Test annotation |

---

## 21. Built-in Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `println!` | `(fmt, args...)` | Print to stdout with newline |
| `print!` | `(fmt, args...)` | Print to stdout without newline |
| `eprintln!` | `(fmt, args...)` | Print to stderr |
| `format!` | `(fmt, args...)` → `string` | Format to string |
| `assert!` | `(condition)` | Assert true, panic otherwise |
| `assert_eq!` | `(a, b)` | Assert equality |
| `panic!` | `(msg)` | Unrecoverable error |
| `todo!` | `()` | Unimplemented marker |
| `unreachable!` | `()` | Dead code marker |
| `vec!` | `(items...)` | Create a Vec |

---

## 22. File Extension

All Ruva source files use the `.ruva` extension.

## 23. Example: Full Program

```ruva
import std::io
import std::fs

#[derive(Debug)]
class Config {
    pub let host: string
    pub let port: u16
    pub let max_connections: u32

    pub fn default() -> Self {
        return Self {
            host: "127.0.0.1".into(),
            port: 8080,
            max_connections: 100,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, string> {
        let raw = fs::read_to_string(path)?
        let config: Config = serde_json::from_str(&raw)
            .map_err(|e| format!("parse error: {}", e))?
        return Ok(config)
    }
}

trait Server {
    fn start(&self) -> Result<(), string>
    fn stop(&self)
    fn is_running(&self) -> bool
}

class HttpServer {
    let config: Config
    let mut running: bool

    pub fn new(config: Config) -> Self {
        return Self { config, running: false }
    }
}

impl Server for HttpServer {
    fn start(&self) -> Result<(), string> {
        println!("Starting server on {}:{}", self.config.host, self.config.port)
        self.running = true
        return Ok(())
    }

    fn stop(&self) {
        println!("Shutting down server...")
        self.running = false
    }

    fn is_running(&self) -> bool {
        return self.running
    }
}

fn main() {
    let config = Config::default()
    let server = HttpServer::new(config)

    try {
        server.start()?
        println!("Server is running!")
        // ... handle connections
    } catch (err) {
        eprintln!("Server failed: {}", err)
    }
}
```
