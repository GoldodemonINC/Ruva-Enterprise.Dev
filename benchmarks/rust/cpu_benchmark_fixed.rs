// CPU Benchmark - Fixed (uses iterative quicksort to avoid stack overflow)
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve, String Processing

use std::time::Instant;

// ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

fn fib_recursive(n: u64) -> u64 {
    if n <= 1 { return n; }
    fib_recursive(n - 1) + fib_recursive(n - 2)
}

fn benchmark_fibonacci() -> f64 {
    let start = Instant::now();
    let mut sum = 0u64;
    for i in 0..40 { sum += fib_recursive(i); }
    std::hint::black_box(sum);
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Fibonacci(40): {} ms", elapsed);
    elapsed
}

// ─── Matrix Multiply ────────────────────────────────────────────────────────

fn benchmark_matrix_multiply() -> f64 {
    let n = 512;
    let mut a = vec![vec![0.0f64; n]; n];
    let mut b = vec![vec![0.0f64; n]; n];
    let mut c = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = (i * j) as f64;
            b[i][j] = (i + j) as f64;
        }
    }
    let start = Instant::now();
    for i in 0..n {
        for j in 0..n {
            let mut s = 0.0f64;
            for k in 0..n { s += a[i][k] * b[k][j]; }
            c[i][j] = s;
        }
    }
    std::hint::black_box(c[0][0]);
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Matrix Multiply (512x512): {} ms", elapsed);
    elapsed
}

// ─── Sorting (Quicksort — iterative to avoid stack overflow) ────────────────

fn partition(arr: &mut [i64], low: isize, high: isize) -> isize {
    let pivot = arr[high as usize];
    let mut i = low - 1;
    for j in low..high {
        if arr[j as usize] < pivot {
            i += 1;
            arr.swap(i as usize, j as usize);
        }
    }
    arr.swap((i + 1) as usize, high as usize);
    i + 1
}

fn benchmark_sorting() -> f64 {
    let n = 1_000_000i64;
    let mut arr: Vec<i64> = (0..n).rev().collect();
    let start = Instant::now();
    // Iterative quicksort with explicit stack
    let mut stack: Vec<(isize, isize)> = Vec::new();
    stack.push((0, (n - 1) as isize));
    while let Some((low, high)) = stack.pop() {
        if low < high {
            let p = partition(&mut arr, low, high);
            if p - 1 > low { stack.push((low, p - 1)); }
            if p + 1 < high { stack.push((p + 1, high)); }
        }
    }
    std::hint::black_box(arr[0]);
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Sorting (1M elements): {} ms", elapsed);
    elapsed
}

// ─── Prime Sieve ────────────────────────────────────────────────────────────

fn benchmark_prime_sieve() -> f64 {
    let limit = 10_000_000usize;
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    let start = Instant::now();
    let mut p = 2;
    while p * p <= limit {
        if sieve[p] {
            let mut i = p * p;
            while i <= limit { sieve[i] = false; i += p; }
        }
        p += 1;
    }
    let count = sieve.iter().filter(|&&x| x).count();
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Prime Sieve ({}): {} primes in {} ms", limit, count, elapsed);
    elapsed
}

// ─── String Processing ──────────────────────────────────────────────────────

fn benchmark_string_processing() -> f64 {
    let n = 100_000usize;
    let start = Instant::now();
    let mut result = String::new();
    for i in 0..n { result.push_str(&format!("Item {}: ", i)); }
    std::hint::black_box(result.len());
    let elapsed = start.elapsed().as_millis() as f64;
    println!("String Concatenation ({}): {} ms", n, elapsed);
    elapsed
}

fn main() {
    println!("=== Rust CPU Benchmark (Fixed) ===\n");
    let mut total = 0.0;
    total += benchmark_fibonacci();
    total += benchmark_matrix_multiply();
    total += benchmark_sorting();
    total += benchmark_prime_sieve();
    total += benchmark_string_processing();
    println!("\nTotal: {} ms", total);
}
