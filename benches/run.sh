#!/bin/bash
# Ruva Transpiler Benchmark Runner
# Builds in release mode and runs the benchmark suite.
#
# Usage:
#   bash benches/run.sh              # Run all benchmarks
#   bash benches/run.sh small        # Run small benchmark only
#   bash benches/run.sh medium       # Run medium benchmark only
#   bash benches/run.sh large        # Run large benchmark only

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building in release mode..."
cd "$PROJECT_DIR"
cargo build --release --quiet 2>/dev/null

TEST_FILTER="${1:-bench_all}"

echo ""
echo "Running benchmarks (filter: $TEST_FILTER)..."
echo ""

cargo test --release --test transpiler_bench -- "$TEST_FILTER" --nocapture 2>/dev/null
