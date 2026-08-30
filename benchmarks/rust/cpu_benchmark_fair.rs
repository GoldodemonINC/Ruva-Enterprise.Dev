// CPU Benchmark - Fair (uses built-in sort, matches real-world usage)
use std::time::Instant;

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

fn benchmark_matrix_multiply() -> f64 {
    let n = 512;
    let mut a = vec![vec![0.0f64; n]; n];
    let mut b = vec![vec![0.0f64; n]; n];
    let mut c = vec![vec![0.0f64; n]; n];
    for i in 0..n { for j in 0..n { a[i][j] = (i*j) as f64; b[i][j] = (i+j) as f64; } }
    let start = Instant::now();
    for i in 0..n { for j in 0..n { let mut s = 0.0; for k in 0..n { s += a[i][k] * b[k][j]; } c[i][j] = s; } }
    std::hint::black_box(c[0][0]);
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Matrix Multiply (512x512): {} ms", elapsed);
    elapsed
}

fn benchmark_sorting() -> f64 {
    let n = 1_000_000i64;
    let mut arr: Vec<i64> = (0..n).rev().collect();
    let start = Instant::now();
    arr.sort_unstable();
    std::hint::black_box(arr[0]);
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Sorting (1M elements): {} ms", elapsed);
    elapsed
}

fn benchmark_prime_sieve() -> f64 {
    let limit = 10_000_000usize;
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false; sieve[1] = false;
    let start = Instant::now();
    let mut p = 2;
    while p * p <= limit { if sieve[p] { let mut i = p * p; while i <= limit { sieve[i] = false; i += p; } } p += 1; }
    let count = sieve.iter().filter(|&&x| x).count();
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Prime Sieve ({}): {} primes in {} ms", limit, count, elapsed);
    elapsed
}

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
    println!("=== Rust CPU Benchmark ===\n");
    let mut total = 0.0;
    total += benchmark_fibonacci();
    total += benchmark_matrix_multiply();
    total += benchmark_sorting();
    total += benchmark_prime_sieve();
    total += benchmark_string_processing();
    println!("\nTotal: {} ms", total);
}
