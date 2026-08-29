#!/bin/bash

# Benchmark Runner - Compiles and runs all benchmarks
# Usage: ./run_benchmarks.sh

set -e

echo "=========================================="
echo "  Ruva vs Rust vs C++ vs Java vs C vs Zig vs Python"
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

# ─── Compile and Run Zig Benchmark ──────────────────────────────────────────

echo "Compiling Zig benchmark..."
cd zig
zig build-exe cpu_benchmark.zig -O ReleaseFast 2>/dev/null || zig cc -O3 cpu_benchmark.zig -o cpu_benchmark 2>/dev/null
echo "Running Zig benchmark..."
./cpu_benchmark > ../results/zig.txt 2>&1 || echo "Zig benchmark skipped (zig not installed)"
cd ..

# ─── Run Python Benchmark ───────────────────────────────────────────────────

echo "Running Python benchmark..."
cd python
python3 cpu_benchmark.py > ../results/python.txt 2>&1 || echo "Python benchmark skipped (python3 not installed)"
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

echo "--- Zig ---"
if [ -f results/zig.txt ]; then
    cat results/zig.txt
else
    echo "Skipped (zig not installed)"
fi
echo ""

echo "--- Python ---"
if [ -f results/python.txt ]; then
    cat results/python.txt
else
    echo "Skipped (python3 not installed)"
fi
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

if [ -f results/zig.txt ]; then
    zig_total=$(grep "Total:" results/zig.txt | awk '{print $2}')
else
    zig_total="N/A"
fi

if [ -f results/python.txt ]; then
    python_total=$(grep "Total:" results/python.txt | awk '{print $2}')
else
    python_total="N/A"
fi

printf "%-11s| %s\n" "Zig" "$zig_total"
printf "%-11s| %s\n" "Python" "$python_total"
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
