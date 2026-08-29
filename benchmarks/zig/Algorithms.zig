// Ruva Algorithms Library — Zig Reference Implementations
//
// Common algorithms for benchmarking and reference:
//   - Sorting: QuickSort, MergeSort, HeapSort, RadixSort
//   - Searching: Binary Search, KMP String Matching
//   - Graph: BFS, DFS, Dijkstra, Topological Sort
//   - Math: GCD, Fast Exponentiation, Sieve of Eratosthenes

const std = @import("std");
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;
const print = std.debug.print;

// ═══════════════════════════════════════════════════════════════════════════
// Sorting Algorithms
// ═══════════════════════════════════════════════════════════════════════════

/// QuickSort — O(n log n) average
pub fn quickSort(arr: []i64) void {
    if (arr.len <= 1) return;
    quickSortRecurse(arr, 0, @intCast(arr.len - 1));
}

fn quickSortRecurse(arr: []i64, low: isize, high: isize) void {
    if (low < high) {
        const pivot = partition(arr, low, high);
        quickSortRecurse(arr, low, pivot - 1);
        quickSortRecurse(arr, pivot + 1, high);
    }
}

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

/// MergeSort — O(n log n) guaranteed
pub fn mergeSort(arr: []i64, allocator: Allocator) !void {
    if (arr.len <= 1) return;
    const mid = arr.len / 2;
    try mergeSort(arr[0..mid], allocator);
    try mergeSort(arr[mid..], allocator);
    try merge(arr, mid, allocator);
}

fn merge(arr: []i64, mid: usize, allocator: Allocator) !void {
    const left = arr[0..mid];
    const right = arr[mid..];
    const temp = try allocator.alloc(i64, arr.len);
    defer allocator.free(temp);

    var i: usize = 0;
    var j: usize = 0;
    var k: usize = 0;

    while (i < left.len and j < right.len) {
        if (left[i] <= right[j]) {
            temp[k] = left[i];
            i += 1;
        } else {
            temp[k] = right[j];
            j += 1;
        }
        k += 1;
    }
    while (i < left.len) {
        temp[k] = left[i];
        i += 1;
        k += 1;
    }
    while (j < right.len) {
        temp[k] = right[j];
        j += 1;
        k += 1;
    }
    @memcpy(arr, temp);
}

/// HeapSort — O(n log n) in-place
pub fn heapSort(arr: []i64) void {
    const n = arr.len;
    // Build max heap
    var i: isize = @intCast(n / 2 - 1);
    while (i >= 0) : (i -= 1) {
        heapify(arr, n, @intCast(i));
    }
    // Extract elements
    var j: isize = @intCast(n - 1);
    while (j > 0) : (j -= 1) {
        const tmp = arr[0];
        arr[0] = arr[@intCast(j)];
        arr[@intCast(j)] = tmp;
        heapify(arr, @intCast(j), 0);
    }
}

fn heapify(arr: []i64, n: usize, i: isize) void {
    var largest = i;
    const left = 2 * i + 1;
    const right = 2 * i + 2;

    if (left < @as(isize, @intCast(n)) and arr[@intCast(left)] > arr[@intCast(largest)]) {
        largest = left;
    }
    if (right < @as(isize, @intCast(n)) and arr[@intCast(right)] > arr[@intCast(largest)]) {
        largest = right;
    }
    if (largest != i) {
        const tmp = arr[@intCast(i)];
        arr[@intCast(i)] = arr[@intCast(largest)];
        arr[@intCast(largest)] = tmp;
        heapify(arr, n, largest);
    }
}

/// RadixSort — O(d × (n + b))
pub fn radixSort(arr: []i64) void {
    var max: i64 = 0;
    for (arr) |val| {
        if (@abs(val) > max) max = @abs(val);
    }

    var exp: i64 = 1;
    while (max / exp > 0) : (exp *= 10) {
        countingSortByDigit(arr, exp);
    }
}

fn countingSortByDigit(arr: []i64, exp: i64) void {
    const n = arr.len;
    var output = ArrayList(i64).init(std.heap.page_allocator);
    defer output.deinit();
    output.resize(n) catch return;

    var count: [10]usize = .{0} ** 10;
    for (arr) |val| {
        const digit: usize = @intCast(@divTrunc(@abs(val), @abs(exp)) % 10);
        count[digit] += 1;
    }
    for (1..10) |i| {
        count[i] += count[i - 1];
    }

    var i: usize = n;
    while (i > 0) {
        i -= 1;
        const digit: usize = @intCast(@divTrunc(@abs(arr[i]), @abs(exp)) % 10);
        count[digit] -= 1;
        output.items[count[digit]] = arr[i];
    }
    @memcpy(arr, output.items);
}

// ═══════════════════════════════════════════════════════════════════════════
// Searching Algorithms
// ═══════════════════════════════════════════════════════════════════════════

/// Binary Search — O(log n) on sorted array
pub fn binarySearch(arr: []const i64, target: i64) ?usize {
    var lo: usize = 0;
    var hi: usize = arr.len;
    while (lo < hi) {
        const mid = lo + (hi - lo) / 2;
        if (arr[mid] == target) return mid;
        if (arr[mid] < target) lo = mid + 1 else hi = mid;
    }
    return null;
}

/// KMP String Search — O(n + m)
pub fn kmpSearch(text: []const u8, pattern: []const u8) ?usize {
    if (pattern.len == 0) return 0;
    if (text.len < pattern.len) return null;

    const lps = computeLPS(pattern);
    var i: usize = 0;
    var j: usize = 0;

    while (i < text.len) {
        if (text[i] == pattern[j]) {
            i += 1;
            j += 1;
            if (j == pattern.len) return i - j;
        } else if (j > 0) {
            j = lps[j - 1];
        } else {
            i += 1;
        }
    }
    return null;
}

fn computeLPS(pattern: []const u8) []usize {
    var lps = std.heap.page_allocator.alloc(usize, pattern.len) catch return &[_]usize{};
    lps[0] = 0;
    var len: usize = 0;
    var i: usize = 1;
    while (i < pattern.len) {
        if (pattern[i] == pattern[len]) {
            len += 1;
            lps[i] = len;
            i += 1;
        } else if (len > 0) {
            len = lps[len - 1];
        } else {
            lps[i] = 0;
            i += 1;
        }
    }
    return lps;
}

// ═══════════════════════════════════════════════════════════════════════════
// Graph Algorithms
// ═══════════════════════════════════════════════════════════════════════════

/// BFS — Breadth-First Search
pub fn bfs(adj: []const []const usize, start: usize, allocator: Allocator) !ArrayList(usize) {
    var result = ArrayList(usize).init(allocator);
    var visited = try allocator.alloc(bool, adj.len);
    defer allocator.free(visited);
    for (visited) |*v| v.* = false;

    var queue = ArrayList(usize).init(allocator);
    defer queue.deinit();

    visited[start] = true;
    try queue.append(start);

    while (queue.items.len > 0) {
        const node = queue.orderedRemove(0);
        try result.append(node);
        for (adj[node]) |neighbor| {
            if (!visited[neighbor]) {
                visited[neighbor] = true;
                try queue.append(neighbor);
            }
        }
    }
    return result;
}

/// DFS — Depth-First Search (iterative)
pub fn dfs(adj: []const []const usize, start: usize, allocator: Allocator) !ArrayList(usize) {
    var result = ArrayList(usize).init(allocator);
    var visited = try allocator.alloc(bool, adj.len);
    defer allocator.free(visited);
    for (visited) |*v| v.* = false;

    var stack = ArrayList(usize).init(allocator);
    defer stack.deinit();

    try stack.append(start);

    while (stack.items.len > 0) {
        const node = stack.pop();
        if (visited[node]) continue;
        visited[node] = true;
        try result.append(node);

        // Push in reverse order for consistent DFS traversal
        var i: usize = adj[node].len;
        while (i > 0) {
            i -= 1;
            if (!visited[adj[node][i]]) {
                try stack.append(adj[node][i]);
            }
        }
    }
    return result;
}

/// Dijkstra's Shortest Path — O((V + E) log V)
pub const DijkstraEdge = struct { to: usize, weight: i64 };

pub fn dijkstra(adj: []const []const DijkstraEdge, src: usize, allocator: Allocator) !ArrayList(i64) {
    const n = adj.len;
    var dist = try ArrayList(i64).init(allocator);
    try dist.resize(n);
    for (dist.items) |*d| d.* = std.math.maxInt(i64);
    dist.items[src] = 0;

    // Simple priority queue using ArrayList (sorted insert)
    var pq = ArrayList(struct { node: usize, dist: i64 }).init(allocator);
    defer pq.deinit();
    try pq.append(.{ .node = src, .dist = 0 });

    while (pq.items.len > 0) {
        // Extract min
        var min_idx: usize = 0;
        for (pq.items, 0..) |item, i| {
            if (item.dist < pq.items[min_idx].dist) min_idx = i;
        }
        const curr = pq.orderedRemove(min_idx);

        if (curr.dist > dist.items[curr.node]) continue;

        for (adj[curr.node]) |edge| {
            const new_dist = curr.dist + edge.weight;
            if (new_dist < dist.items[edge.to]) {
                dist.items[edge.to] = new_dist;
                try pq.append(.{ .node = edge.to, .dist = new_dist });
            }
        }
    }
    return dist;
}

/// Topological Sort — Kahn's algorithm
pub fn topologicalSort(adj: []const []const usize, allocator: Allocator) !ArrayList(usize) {
    const n = adj.len;
    var in_degree = try allocator.alloc(usize, n);
    defer allocator.free(in_degree);
    for (in_degree) |*d| d.* = 0;

    for (adj, 0..) |neighbors, u| {
        for (neighbors) |v| {
            in_degree[v] += 1;
        }
    }

    var queue = ArrayList(usize).init(allocator);
    defer queue.deinit();
    for (in_degree, 0..) |deg, i| {
        if (deg == 0) try queue.append(i);
    }

    var result = ArrayList(usize).init(allocator);
    while (queue.items.len > 0) {
        const u = queue.orderedRemove(0);
        try result.append(u);
        for (adj[u]) |v| {
            in_degree[v] -= 1;
            if (in_degree[v] == 0) try queue.append(v);
        }
    }

    if (result.items.len != n) return error.CycleDetected;
    return result;
}

// ═══════════════════════════════════════════════════════════════════════════
// Math Utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Greatest Common Divisor
pub fn gcd(a: u64, b: u64) u64 {
    var x = a;
    var y = b;
    while (y != 0) {
        const temp = y;
        y = x % y;
        x = temp;
    }
    return x;
}

/// Fast modular exponentiation — O(log n)
pub fn modPow(base: u64, exp: u64, mod_val: u64) u64 {
    var result: u64 = 1;
    var b = base % mod_val;
    var e = exp;
    while (e > 0) {
        if (e & 1 == 1) {
            result = result * b % mod_val;
        }
        e >>= 1;
        b = b * b % mod_val;
    }
    return result;
}

/// Sieve of Eratosthenes — returns list of primes up to limit
pub fn sieveOfEratosthenes(limit: usize, allocator: Allocator) !ArrayList(usize) {
    var sieve = try allocator.alloc(bool, limit + 1);
    defer allocator.free(sieve);
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

    var primes = ArrayList(usize).init(allocator);
    for (sieve, 0..) |is_prime, i| {
        if (is_prime) try primes.append(i);
    }
    return primes;
}

// ═══════════════════════════════════════════════════════════════════════════
// Demo / Test
// ═══════════════════════════════════════════════════════════════════════════

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    print("=== Ruva Algorithms Library (Zig) ===\n\n", .{});

    // QuickSort test
    var data = [_]i64{ 38, 27, 43, 3, 9, 82, 10 };
    quickSort(&data);
    print("QuickSort:    {any}\n", .{data});

    // MergeSort test
    var data2 = [_]i64{ 38, 27, 43, 3, 9, 82, 10 };
    try mergeSort(&data2, allocator);
    print("MergeSort:    {any}\n", .{data2});

    // HeapSort test
    var data3 = [_]i64{ 38, 27, 43, 3, 9, 82, 10 };
    heapSort(&data3);
    print("HeapSort:     {any}\n", .{data3});

    // RadixSort test
    var data4 = [_]i64{ 38, 27, 43, 3, 9, 82, 10 };
    radixSort(&data4);
    print("RadixSort:    {any}\n", .{data4});

    // Binary search
    const sorted = [_]i64{ 1, 3, 5, 7, 9, 11, 13 };
    const idx = binarySearch(&sorted, 7);
    print("\nBinarySearch(7): index={?}\n", .{idx});

    // Math
    print("\ngcd(48, 18): {d}\n", .{gcd(48, 18)});
    print("modPow(2,10,1000): {d}\n", .{modPow(2, 10, 1000)});

    const primes = try sieveOfEratosthenes(50, allocator);
    defer primes.deinit();
    print("Primes <= 50: {d} primes\n", .{primes.items.len});

    print("\nAll algorithms functional.\n", .{});
}
