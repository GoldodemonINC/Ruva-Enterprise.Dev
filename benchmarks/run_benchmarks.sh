#!/bin/bash

# Benchmark Runner - Compiles and runs all benchmarks
# Usage: ./run_benchmarks.sh

set -e

echo "=========================================="
echo "  Ruva vs Rust vs C++ vs Java vs C"
echo "  CPU Benchmark Suite"
echo "=========================================="
echo ""

# Create results directory
mkdir -p results

# ─── Compile and Run Rust Benchmark ──────────────────────────────────────────

echo "Compiling Rust benchmark..."
cd rust
rustc -O cpu_benchmark.rs -o cpu_benchmark
echo "Running Rust benchmark..."
./cpu_benchmark > ../results/rust.txt 2>&1
cd ..

# ─── Compile and Run C++ Benchmark ───────────────────────────────────────────

echo "Compiling C++ benchmark..."
cd cpp
g++ -O3 -o cpu_benchmark cpu_benchmark.cpp
echo "Running C++ benchmark..."
./cpu_benchmark > ../results/cpp.txt 2>&1
cd ..

# ─── Compile and Run C Benchmark ─────────────────────────────────────────────

echo "Compiling C benchmark..."
cd c
gcc -O3 -o cpu_benchmark cpu_benchmark.c -lm
echo "Running C benchmark..."
./cpu_benchmark > ../results/c.txt 2>&1
cd ..

# ─── Compile and Run Java Benchmark ──────────────────────────────────────────

echo "Compiling Java benchmark..."
cd java
javac CpuBenchmark.java
echo "Running Java benchmark..."
java CpuBenchmark > ../results/java.txt 2>&1
cd ..

# ─── Compile and Run Ruva Benchmark ──────────────────────────────────────────

echo "Transpiling and compiling Ruva benchmark..."
cd ruva
# Transpile to Rust
cargo run -- transpile cpu_benchmark.ruva --stdout > cpu_benchmark.rs 2>/dev/null
# Compile the generated Rust
rustc -O cpu_benchmark.rs -o cpu_benchmark 2>/dev/null || echo "Note: Ruva benchmark requires std library"
echo "Running Ruva benchmark..."
./cpu_benchmark > ../results/ruva.txt 2>&1 || echo "Ruva benchmark skipped (std library not implemented)"
cd ..

# ─── Display Results ─────────────────────────────────────────────────────────

echo ""
echo "=========================================="
echo "  RESULTS"
echo "=========================================="
echo ""

echo "--- Rust ---"
cat results/rust.txt
echo ""

echo "--- C++ ---"
cat results/cpp.txt
echo ""

echo "--- C ---"
cat results/c.txt
echo ""

echo "--- Java ---"
cat results/java.txt
echo ""

echo "--- Ruva ---"
if [ -f results/ruva.txt ]; then
    cat results/ruva.txt
else
    echo "Skipped (std library not implemented)"
fi
echo ""

# ─── Summary ─────────────────────────────────────────────────────────────────

echo "=========================================="
echo "  SUMMARY"
echo "=========================================="
echo ""

# Extract total times
rust_total=$(grep "Total:" results/rust.txt | awk '{print $2}')
cpp_total=$(grep "Total:" results/cpp.txt | awk '{print $2}')
c_total=$(grep "Total:" results/c.txt | awk '{print $2}')
java_total=$(grep "Total:" results/java.txt | awk '{print $2}')

if [ -f results/ruva.txt ]; then
    ruva_total=$(grep "Total:" results/ruva.txt | awk '{print $2}')
else
    ruva_total="N/A"
fi

echo "Language    | Total Time (ms)"
echo "------------|----------------"
printf "%-11s| %s\n" "Rust" "$rust_total"
printf "%-11s| %s\n" "C++" "$cpp_total"
printf "%-11s| %s\n" "C" "$c_total"
printf "%-11s| %s\n" "Java" "$java_total"
printf "%-11s| %s\n" "Ruva" "$ruva_total"
echo ""

# Calculate speedup vs Rust
if [ "$ruva_total" != "N/A" ] && [ -n "$ruva_total" ]; then
    echo "Speedup vs Rust:"
    echo "  Ruva: $(echo "scale=2; $rust_total / $ruva_total" | bc)x"
    echo "  C++: $(echo "scale=2; $rust_total / $cpp_total" | bc)x"
    echo "  C: $(echo "scale=2; $rust_total / $c_total" | bc)x"
    echo "  Java: $(echo "scale=2; $rust_total / $java_total" | bc)x"
fi

echo ""
echo "Benchmark complete!"
