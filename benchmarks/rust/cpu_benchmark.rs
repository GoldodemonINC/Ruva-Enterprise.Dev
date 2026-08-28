// CPU Benchmark - Rust
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve

use std::time::Instant;

// ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

fn fib_recursive(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    fib_recursive(n - 1) + fib_recursive(n - 2)
}

fn benchmark_fibonacci() -> f64 {
    let start = Instant::now();
    
    let mut sum = 0u64;
    for i in 0..40 {
        sum += fib_recursive(i);
    }
    
    let elapsed = start.elapsed().as_millis() as f64;
    println!("Fibonacci(40): {} ms", elapsed);
    elapsed
}

// ─── Matrix Multiply ────────────────────────────────────────────────────────

fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>], c: &mut [Vec<f64>], n: usize) {
    for i in 0..n {
        for j in 0..n {
            c[i][j] = 0.0;
            for k in 0..n {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

fn benchmark_matrix_multiply() -> f64 {
    let n = 512;
    let mut a = vec![vec![0.0; n]; n];
    let mut b = vec![vec![0.0; n]; n];
    let mut c = vec![vec![0.0; n]; n];
    
    // Initialize matrices
    for i in 0..n {
        for j in 0..n {
            a[i][j] = (i * j) as f64;
            b[i][j] = (i + j) as f64;
        }
    }
    
    let start = Instant::now();
    matrix_multiply(&a, &b, &mut c, n);
    let elapsed = start.elapsed().as_millis() as f64;
    
    println!("Matrix Multiply (512x512): {} ms", elapsed);
    elapsed
}

// ─── Sorting (Quicksort) ────────────────────────────────────────────────────

fn quicksort(arr: &mut [i64], low: isize, high: isize) {
    if low < high {
        let pivot = partition(arr, low, high);
        quicksort(arr, low, pivot - 1);
        quicksort(arr, pivot + 1, high);
    }
}

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
    let n = 1_000_000;
    let mut arr: Vec<i64> = (0..n).rev().collect();
    
    let start = Instant::now();
    quicksort(&mut arr, 0, (n - 1) as isize);
    let elapsed = start.elapsed().as_millis() as f64;
    
    println!("Sorting (1M elements): {} ms", elapsed);
    elapsed
}

// ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────────

fn sieve_of_eratosthenes(limit: usize) -> usize {
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    
    let mut p = 2;
    while p * p <= limit {
        if sieve[p] {
            let mut i = p * p;
            while i <= limit {
                sieve[i] = false;
                i += p;
            }
        }
        p += 1;
    }
    
    sieve.iter().filter(|&&x| x).count()
}

fn benchmark_prime_sieve() -> f64 {
    let limit = 10_000_000;
    
    let start = Instant::now();
    let count = sieve_of_eratosthenes(limit);
    let elapsed = start.elapsed().as_millis() as f64;
    
    println!("Prime Sieve ({}): {} primes in {} ms", limit, count, elapsed);
    elapsed
}

// ─── String Processing ──────────────────────────────────────────────────────

fn string_concatenation(n: usize) -> String {
    let mut result = String::new();
    for i in 0..n {
        result.push_str(&format!("Item {}: ", i));
    }
    result
}

fn benchmark_string_processing() -> f64 {
    let n = 100_000;
    
    let start = Instant::now();
    let _result = string_concatenation(n);
    let elapsed = start.elapsed().as_millis() as f64;
    
    println!("String Concatenation ({}): {} ms", n, elapsed);
    elapsed
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Rust CPU Benchmark ===");
    println!();
    
    let mut total = 0.0;
    
    total += benchmark_fibonacci();
    total += benchmark_matrix_multiply();
    total += benchmark_sorting();
    total += benchmark_prime_sieve();
    total += benchmark_string_processing();
    
    println!();
    println!("Total: {} ms", total);
}
