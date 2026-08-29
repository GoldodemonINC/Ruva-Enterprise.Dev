// CPU Benchmark - Zig
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve

const std = @import("std");

// ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

fn fibRecursive(n: u64) u64 {
    if (n <= 1) return n;
    return fibRecursive(n - 1) + fibRecursive(n - 2);
}

fn benchmarkFibonacci() f64 {
    const start = std.time.milliTimestamp();

    var sum: u64 = 0;
    var i: u64 = 0;
    while (i < 40) : (i += 1) {
        sum += fibRecursive(i);
    }

    const elapsed: f64 = @floatFromInt(std.time.milliTimestamp() - start);
    std.debug.print("Fibonacci(40): {d} ms\n", .{elapsed});
    return elapsed;
}

// ─── Matrix Multiply ────────────────────────────────────────────────────────

fn matrixMultiply(a: [][]f64, b: [][]f64, c: [][]f64, n: usize) void {
    for (0..n) |i| {
        for (0..n) |j| {
            c[i][j] = 0.0;
            for (0..n) |k| {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

fn benchmarkMatrixMultiply() f64 {
    const n: usize = 512;
    const allocator = std.heap.page_allocator;

    var a = allocator.alloc([]f64, n) catch unreachable;
    var b = allocator.alloc([]f64, n) catch unreachable;
    var c = allocator.alloc([]f64, n) catch unreachable;

    for (0..n) |i| {
        a[i] = allocator.alloc(f64, n) catch unreachable;
        b[i] = allocator.alloc(f64, n) catch unreachable;
        c[i] = allocator.alloc(f64, n) catch unreachable;
        for (0..n) |j| {
            a[i][j] = @floatFromInt(i * j);
            b[i][j] = @floatFromInt(i + j);
        }
    }

    const start = std.time.milliTimestamp();
    matrixMultiply(a, b, c, n);
    const elapsed: f64 = @floatFromInt(std.time.milliTimestamp() - start);

    std.debug.print("Matrix Multiply (512x512): {d} ms\n", .{elapsed});

    for (0..n) |i| {
        allocator.free(a[i]);
        allocator.free(b[i]);
        allocator.free(c[i]);
    }
    allocator.free(a);
    allocator.free(b);
    allocator.free(c);

    return elapsed;
}

// ─── Sorting (Quicksort) ────────────────────────────────────────────────────

fn partition(arr: []i64, low: isize, high: isize) isize {
    const pivot = arr[@intCast(high)];
    var i: isize = low - 1;
    var j: isize = low;

    while (j < high) : (j += 1) {
        if (arr[@intCast(j)] < pivot) {
            i += 1;
            const tmp = arr[@intCast(i)];
            arr[@intCast(i)] = arr[@intCast(j)];
            arr[@intCast(j)] = tmp;
        }
    }
    const tmp = arr[@intCast(i + 1)];
    arr[@intCast(i + 1)] = arr[@intCast(high)];
    arr[@intCast(high)] = tmp;
    return i + 1;
}

fn quicksort(arr: []i64, low: isize, high: isize) void {
    if (low < high) {
        const pi = partition(arr, low, high);
        quicksort(arr, low, pi - 1);
        quicksort(arr, pi + 1, high);
    }
}

fn benchmarkSorting() f64 {
    const n: usize = 1_000_000;
    const allocator = std.heap.page_allocator;

    var arr = allocator.alloc(i64, n) catch unreachable;
    var idx: usize = 0;
    while (idx < n) : (idx += 1) {
        arr[idx] = @intCast(n - idx - 1);
    }

    const start = std.time.milliTimestamp();
    quicksort(arr, 0, @intCast(n - 1));
    const elapsed: f64 = @floatFromInt(std.time.milliTimestamp() - start);

    std.debug.print("Sorting (1M elements): {d} ms\n", .{elapsed});
    allocator.free(arr);
    return elapsed;
}

// ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────────

fn sieveOfEratosthenes(limit: usize) usize {
    const allocator = std.heap.page_allocator;
    var sieve = allocator.alloc(bool, limit + 1) catch unreachable;
    for (sieve) |*s| s.* = true;
    sieve[0] = false;
    sieve[1] = false;

    var p: usize = 2;
    while (p * p <= limit) : (p += 1) {
        if (sieve[p]) {
            var i: usize = p * p;
            while (i <= limit) : (i += p) {
                sieve[i] = false;
            }
        }
    }

    var count: usize = 0;
    for (sieve) |s| {
        if (s) count += 1;
    }
    allocator.free(sieve);
    return count;
}

fn benchmarkPrimeSieve() f64 {
    const limit: usize = 10_000_000;

    const start = std.time.milliTimestamp();
    const count = sieveOfEratosthenes(limit);
    const elapsed: f64 = @floatFromInt(std.time.milliTimestamp() - start);

    std.debug.print("Prime Sieve ({d}): {d} primes in {d} ms\n", .{ limit, count, elapsed });
    return elapsed;
}

// ─── String Processing ──────────────────────────────────────────────────────

fn stringConcatenation(n: usize) ![]u8 {
    const allocator = std.heap.page_allocator;
    var result = std.ArrayList(u8).init(allocator);
    errdefer result.deinit();

    var i: usize = 0;
    while (i < n) : (i += 1) {
        const item = try std.fmt.allocPrint(allocator, "Item {d} ", .{i});
        defer allocator.free(item);
        try result.appendSlice(item);
    }

    return result.items;
}

fn benchmarkStringProcessing() f64 {
    const n: usize = 100_000;

    const start = std.time.milliTimestamp();
    const result = stringConcatenation(n) catch unreachable;
    const elapsed: f64 = @floatFromInt(std.time.milliTimestamp() - start);

    std.debug.print("String Concatenation ({d}): {d} ms\n", .{ n, elapsed });
    _ = result;
    return elapsed;
}

// ─── Main ───────────────────────────────────────────────────────────────────

pub fn main() void {
    std.debug.print("=== Zig CPU Benchmark ===\n\n", .{});

    var total: f64 = 0;

    total += benchmarkFibonacci();
    total += benchmarkMatrixMultiply();
    total += benchmarkSorting();
    total += benchmarkPrimeSieve();
    total += benchmarkStringProcessing();

    std.debug.print("\nTotal: {d} ms\n", .{total});
}
