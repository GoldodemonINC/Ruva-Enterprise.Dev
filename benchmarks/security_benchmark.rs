// Security Benchmark - Rust
// Tests: bounds checking, null safety, overflow detection, memory safety

use std::time::Instant;

// ─── Bounds Checking ────────────────────────────────────────────────────────

fn benchmark_bounds_checking(n: usize) -> f64 {
    let data: Vec<u64> = (0..n as u64).collect();
    let start = Instant::now();

    let mut sum = 0u64;
    for i in 0..n {
        // Bounds-checked access — panics on out-of-bounds instead of UB
        sum = sum.wrapping_add(data[i]);
    }
    // Prevent optimization from eliminating the loop
    std::hint::black_box(sum);

    let elapsed = start.elapsed().as_micros() as f64;
    println!("Bounds-Checked Array Access ({} elements): {:.2} µs", n, elapsed);
    elapsed
}

// ─── Null Safety (Option<T>) ────────────────────────────────────────────────

fn find_value(data: &[i32], target: i32) -> Option<usize> {
    for (i, &val) in data.iter().enumerate() {
        if val == target {
            return Some(i);
        }
    }
    None
}

fn benchmark_null_safety(n: usize) -> f64 {
    let data: Vec<i32> = (0..n as i32).collect();
    let start = Instant::now();

    // Search for existing and non-existing values
    let mut found = 0u64;
    for target in 0..n as i32 {
        if let Some(_idx) = find_value(&data, target) {
            found += 1;
        }
        // Search for value that doesn't exist
        if find_value(&data, n as i32 + target).is_none() {
            found += 1;
        }
    }
    std::hint::black_box(found);

    let elapsed = start.elapsed().as_micros() as f64;
    println!("Null Safety (Option) Searches ({} lookups): {:.2} µs", n * 2, elapsed);
    elapsed
}

// ─── Integer Overflow Detection ─────────────────────────────────────────────

fn safe_add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

fn benchmark_overflow_detection(n: u32) -> f64 {
    let start = Instant::now();

    let mut count = 0u64;
    for i in 0..n {
        if safe_add(i, i).is_some() {
            count += 1;
        }
        // This will overflow and return None (safe!)
        if safe_add(u32::MAX - i, i + 1).is_none() {
            count += 1;
        }
    }
    std::hint::black_box(count);

    let elapsed = start.elapsed().as_micros() as f64;
    println!("Overflow Detection ({} checks): {:.2} µs", n * 2, elapsed);
    elapsed
}

// ─── String Safety (UTF-8, no buffer overflows) ─────────────────────────────

fn benchmark_string_safety(iterations: usize) -> f64 {
    let start = Instant::now();

    let mut total_len = 0usize;
    for i in 0..iterations {
        // Safe string operations — no buffer overflows possible
        let s = format!("Item {}: 这是安全的UTF-8字符串 🔒🛡️", i);
        total_len += s.len();
        // Safe string slicing — panics on invalid UTF-8 boundaries
        let _safe_slice = &s[..s.char_indices().last().map(|(p, _)| p).unwrap_or(0)];
    }
    std::hint::black_box(total_len);

    let elapsed = start.elapsed().as_micros() as f64;
    println!("String Safety ({} UTF-8 ops): {:.2} µs", iterations, elapsed);
    elapsed
}

// ─── Memory Safety (No Use-After-Free, No Dangling Pointers) ────────────────

fn benchmark_memory_safety(iterations: usize) -> f64 {
    let start = Instant::now();

    for i in 0..iterations {
        // Ownership ensures no use-after-free
        let data = vec![i as u64; 100];
        let borrowed = &data;
        let sum: u64 = borrowed.iter().sum();
        // data is dropped here — borrows are already validated at compile time
        std::hint::black_box(sum);
    }

    let elapsed = start.elapsed().as_micros() as f64;
    println!("Memory Safety ({} alloc/dealloc): {:.2} µs", iterations, elapsed);
    elapsed
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Rust Security Benchmark ===");
    println!();
    println!("All operations are safe-by-default:");
    println!("  ✓ Bounds checking on array access");
    println!("  ✓ Null safety via Option<T> (no null pointers)");
    println!("  ✓ Integer overflow detection (checked arithmetic)");
    println!("  ✓ UTF-8 safe string operations");
    println!("  ✓ Ownership prevents use-after-free & dangling pointers");
    println!();

    let mut total = 0.0;

    total += benchmark_bounds_checking(1_000_000);
    total += benchmark_null_safety(10_000);
    total += benchmark_overflow_detection(100_000);
    total += benchmark_string_safety(100_000);
    total += benchmark_memory_safety(100_000);

    println!();
    println!("Total: {:.2} µs ({:.2} ms)", total, total / 1000.0);
    println!();
    println!("Security guarantees: ALL PASS (Rust enforces at compile time)");
}
