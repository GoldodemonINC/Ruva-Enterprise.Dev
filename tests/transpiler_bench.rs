//! Transpiler benchmark suite.
//!
//! Measures timing and memory usage for lexing, parsing, type checking,
//! and Rust code generation across three input sizes.
//!
//! Memory is tracked via process RSS (Resident Set Size) measured before
//! and after each pipeline stage.
//!
//! Run with:  cargo test --release --test transpiler_bench -- --nocapture

use ruva::backend::CodeGenerator;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Resolve a source file that may be named `.rve` or `.ruva`, so the harness works
/// both before and after the extension rename.
fn input_path(name: &str) -> PathBuf {
    let dir = project_root().join("benches/inputs");
    for ext in [".rve", ".ruva"] {
        let p = dir.join(format!("{}{}", name, ext));
        if p.exists() { return p; }
    }
    dir.join(format!("{}.ruva", name))
}

// Timing Infrastructure

struct BenchResult {
    name: String,
    loc: usize,
    iterations: usize,
    lex_us: Vec<u64>,
    parse_us: Vec<u64>,
    typecheck_us: Vec<u64>,
    codegen_us: Vec<u64>,
    // Memory: RSS delta (bytes) for each stage, collected per iteration
    lex_mem: Vec<usize>,
    parse_mem: Vec<usize>,
    typecheck_mem: Vec<usize>,
    codegen_mem: Vec<usize>,
    // Peak RSS at end of each iteration (for total)
    peak_rss: Vec<usize>,
}

fn median_us(times: &[u64]) -> f64 {
    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
    } else {
        sorted[mid] as f64
    }
}

fn mean_us(times: &[u64]) -> f64 {
    times.iter().sum::<u64>() as f64 / times.len() as f64
}

fn format_duration(us: f64) -> String {
    if us < 1_000.0 {
        format!("{:.1} µs", us)
    } else if us < 1_000_000.0 {
        format!("{:.2} ms", us / 1_000.0)
    } else {
        format!("{:.2} s", us / 1_000_000.0)
    }
}

// Memory Measurement

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global allocation counters, updated by the custom allocator.
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOC_PEAK: AtomicUsize = AtomicUsize::new(0);

/// Custom global allocator that tracks allocation counts and bytes.
struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let bytes = layout.size();
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(bytes, Ordering::Relaxed);
            // Update peak: compare-and-swap loop
            loop {
                let peak = ALLOC_PEAK.load(Ordering::Relaxed);
                let current = ALLOC_BYTES.load(Ordering::Relaxed);
                if current <= peak {
                    break;
                }
                if ALLOC_PEAK.compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                    break;
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOC_COUNT.fetch_sub(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

/// Snapshot of allocation counters at a point in time.
#[derive(Clone, Copy)]
struct AllocSnapshot {
    count: usize,
    bytes: usize,
    peak: usize,
}

fn snapshot() -> AllocSnapshot {
    AllocSnapshot {
        count: ALLOC_COUNT.load(Ordering::Relaxed),
        bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        peak: ALLOC_PEAK.load(Ordering::Relaxed),
    }
}

fn delta_bytes(before: AllocSnapshot, after: AllocSnapshot) -> usize {
    after.bytes.saturating_sub(before.bytes)
}

/// Format bytes into a human-readable string.
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// Single-iteration benchmark

fn bench_one_iteration(source: &str) -> (u64, u64, u64, u64, usize, usize, usize, usize, usize) {
    // Lex
    let snap_before = snapshot();
    let t0 = Instant::now();
    let mut lexer = ruva::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let lex_us = t0.elapsed().as_micros() as u64;
    black_box(&tokens);
    let lex_mem = delta_bytes(snap_before, snapshot());

    // Parse
    let snap_before = snapshot();
    let t1 = Instant::now();
    let mut parser = ruva::parser::Parser::new(source).unwrap();
    let program = parser.parse_program().unwrap();
    let parse_us = t1.elapsed().as_micros() as u64;
    black_box(&program);
    let parse_mem = delta_bytes(snap_before, snapshot());

    // Type check
    let snap_before = snapshot();
    let t2 = Instant::now();
    let mut checker = ruva::typecheck::TypeChecker::new();
    let diagnostics = checker.check(&program);
    let typecheck_us = t2.elapsed().as_micros() as u64;
    black_box(&diagnostics);
    let typecheck_mem = delta_bytes(snap_before, snapshot());

    // Codegen (Rust backend)
    let snap_before = snapshot();
    let t3 = Instant::now();
    let mut gen = ruva::codegen::CodeGen::new();
    let code = gen.generate(&program);
    let codegen_us = t3.elapsed().as_micros() as u64;
    black_box(&code);
    let codegen_mem = delta_bytes(snap_before, snapshot());

    let peak = snapshot().peak;

    (lex_us, parse_us, typecheck_us, codegen_us,
     lex_mem, parse_mem, typecheck_mem, codegen_mem, peak)
}

// Run full benchmark for one input

fn run_bench(name: &str, iterations: usize) -> BenchResult {
    let source = fs::read_to_string(input_path(name))
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", name, e));
    let loc = source.lines().count();

    // Warmup
    for _ in 0..3 {
        black_box(bench_one_iteration(&source));
    }

    let mut lex_us = Vec::with_capacity(iterations);
    let mut parse_us = Vec::with_capacity(iterations);
    let mut typecheck_us = Vec::with_capacity(iterations);
    let mut codegen_us = Vec::with_capacity(iterations);
    let mut lex_mem = Vec::with_capacity(iterations);
    let mut parse_mem = Vec::with_capacity(iterations);
    let mut typecheck_mem = Vec::with_capacity(iterations);
    let mut codegen_mem = Vec::with_capacity(iterations);
    let mut peak_rss = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let (l, p, t, c, lm, pm, tm, cm, pr) = bench_one_iteration(&source);
        lex_us.push(l);
        parse_us.push(p);
        typecheck_us.push(t);
        codegen_us.push(c);
        lex_mem.push(lm);
        parse_mem.push(pm);
        typecheck_mem.push(tm);
        codegen_mem.push(cm);
        peak_rss.push(pr);
    }

    BenchResult {
        name: name.to_string(),
        loc,
        iterations,
        lex_us,
        parse_us,
        typecheck_us,
        codegen_us,
        lex_mem,
        parse_mem,
        typecheck_mem,
        codegen_mem,
        peak_rss,
    }
}

// Report Printing

fn median_bytes(values: &[usize]) -> usize {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2
    } else {
        sorted[mid]
    }
}

fn mean_bytes(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len() as f64
}

fn print_report(results: &[BenchResult]) {
    let sep = "─".repeat(95);
    println!();
    println!("  Ruva Transpiler Benchmark Report");
    println!("  {}", sep);
    println!();

    for r in results {
        let total_median: f64 = median_us(&r.lex_us)
            + median_us(&r.parse_us)
            + median_us(&r.typecheck_us)
            + median_us(&r.codegen_us);

        println!("  {}  ({} LOC, {} iterations)", r.name, r.loc, r.iterations);
        println!();

        // Timing section
        println!("  {:>18}  median {:>12}   mean {:>12}", "Lexing", format_duration(median_us(&r.lex_us)), format_duration(mean_us(&r.lex_us)));
        println!("  {:>18}  median {:>12}   mean {:>12}", "Parsing", format_duration(median_us(&r.parse_us)), format_duration(mean_us(&r.parse_us)));
        println!("  {:>18}  median {:>12}   mean {:>12}", "Type checking", format_duration(median_us(&r.typecheck_us)), format_duration(mean_us(&r.typecheck_us)));
        println!("  {:>18}  median {:>12}   mean {:>12}", "Codegen (Rust)", format_duration(median_us(&r.codegen_us)), format_duration(mean_us(&r.codegen_us)));
        println!("  {:>18}  ──────────────────────", "");
        println!("  {:>18}  median {:>12}", "TOTAL", format_duration(total_median));
        println!("  {} us/sec (parsing only)", (r.loc as f64 / median_us(&r.parse_us) * 1_000_000.0) as u64);
        println!();

        // Memory section
        println!("  {:>18}  {:>12}  {:>12}", "Memory (heap)", "median", "mean");
        println!("  {:>18}  {:>12}  {:>12}", "Lexing", format_bytes(median_bytes(&r.lex_mem)), format_bytes(mean_bytes(&r.lex_mem) as usize));
        println!("  {:>18}  {:>12}  {:>12}", "Parsing", format_bytes(median_bytes(&r.parse_mem)), format_bytes(mean_bytes(&r.parse_mem) as usize));
        println!("  {:>18}  {:>12}  {:>12}", "Type checking", format_bytes(median_bytes(&r.typecheck_mem)), format_bytes(mean_bytes(&r.typecheck_mem) as usize));
        println!("  {:>18}  {:>12}  {:>12}", "Codegen (Rust)", format_bytes(median_bytes(&r.codegen_mem)), format_bytes(mean_bytes(&r.codegen_mem) as usize));
        println!("  {:>18}  {:>12}", "Peak heap", format_bytes(median_bytes(&r.peak_rss)));
        println!("  {:>18}  heap/LOC {:>6}", "", format!("{:.0} B", median_bytes(&r.peak_rss) as f64 / r.loc as f64));
        println!();
    }

    // Throughput table
    println!("  {}", sep);
    println!("  Throughput Summary");
    println!("  {:>20}  {:>12}  {:>12}  {:>12}  {:>12}", "Input", "Lex", "Parse", "TypeCheck", "Codegen");
    println!("  {:>20}  {:>12}  {:>12}  {:>12}  {:>12}", "", "(LOC/sec)", "(LOC/sec)", "(LOC/sec)", "(LOC/sec)");
    for r in results {
        let lex_throughput = r.loc as f64 / median_us(&r.lex_us) * 1_000_000.0;
        let parse_throughput = r.loc as f64 / median_us(&r.parse_us) * 1_000_000.0;
        let tc_throughput = r.loc as f64 / median_us(&r.typecheck_us) * 1_000_000.0;
        let cg_throughput = r.loc as f64 / median_us(&r.codegen_us) * 1_000_000.0;
        println!(
            "  {:>20}  {:>10.0}/s  {:>10.0}/s  {:>10.0}/s  {:>10.0}/s",
            format!("{} ({} LOC)", r.name, r.loc),
            lex_throughput, parse_throughput, tc_throughput, cg_throughput
        );
    }
    println!();

    // Memory summary table
    println!("  Memory Summary (peak heap allocation)");
    println!("  {:>20}  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}", "Input", "Lexing", "Parsing", "TypeCheck", "Codegen", "Peak Heap");
    for r in results {
        println!(
            "  {:>20}  {:>12}  {:>12}  {:>12}  {:>12}  {:>12}",
            format!("{} ({} LOC)", r.name, r.loc),
            format_bytes(median_bytes(&r.lex_mem)),
            format_bytes(median_bytes(&r.parse_mem)),
            format_bytes(median_bytes(&r.typecheck_mem)),
            format_bytes(median_bytes(&r.codegen_mem)),
            format_bytes(median_bytes(&r.peak_rss)),
        );
    }
    println!("  {}", sep);
    println!();
}

// Tests

#[test]
fn bench_small() {
    let r = run_bench("small", 100);
    print_report(&[r]);
}

#[test]
fn bench_medium() {
    let r = run_bench("medium", 50);
    print_report(&[r]);
}

#[test]
fn bench_large() {
    let r = run_bench("large", 20);
    print_report(&[r]);
}

#[test]
fn bench_all() {
    println!("\n  Running transpiler benchmarks...\n");

    let results = vec![
        run_bench("small", 100),
        run_bench("medium", 50),
        run_bench("large", 20),
    ];

    print_report(&results);
}
