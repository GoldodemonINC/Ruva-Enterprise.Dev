// Ruva Data Structures Library — Zig Reference Implementations
//
// Core data structures for benchmarking and reference:
//   - ArrayList (dynamic array)
//   - HashMap (hash table with open addressing)
//   - Stack (LIFO)
//   - Queue (circular buffer)
//   - BinarySearchTree
//   - MinHeap
//   - UnionFind

const std = @import("std");
const Allocator = std.mem.Allocator;
const print = std.debug.print;
const expect = std.testing.expect;

// ═══════════════════════════════════════════════════════════════════════════
// ArrayList — Dynamic array with automatic resizing
// ═══════════════════════════════════════════════════════════════════════════

pub fn ArrayList(comptime T: type) type {
    return struct {
        items: []T,
        len: usize,
        capacity: usize,
        allocator: Allocator,

        pub const empty: @This() = .{
            .items = &.{},
            .len = 0,
            .capacity = 0,
            .allocator = undefined,
        };

        pub fn init(allocator: Allocator) @This() {
            return .{
                .items = &.{},
                .len = 0,
                .capacity = 0,
                .allocator = allocator,
            };
        }

        pub fn deinit(self: *@This()) void {
            self.allocator.free(self.items.ptr[0..self.capacity]);
            self.* = empty;
        }

        pub fn ensureTotalCapacity(self: *@This(), new_capacity: usize) !void {
            if (self.capacity >= new_capacity) return;
            const new_mem = try self.allocator.realloc(self.items.ptr[0..self.capacity], new_capacity);
            self.items.ptr = new_mem.ptr;
            self.capacity = new_capacity;
        }

        pub fn append(self: *@This(), item: T) !void {
            try self.ensureTotalCapacity(@max(self.capacity, 1) * 2);
            self.items.ptr[self.len] = item;
            self.len += 1;
        }

        pub fn appendSlice(self: *@This(), slice: []const T) !void {
            for (slice) |item| {
                try self.append(item);
            }
        }

        pub fn itemsSlice(self: *const @This()) []T {
            return self.items.ptr[0..self.len];
        }

        pub fn get(self: *const @This(), index: usize) ?T {
            if (index >= self.len) return null;
            return self.items.ptr[index];
        }

        pub fn set(self: *@This(), index: usize, item: T) !void {
            if (index >= self.len) return error.IndexOutOfBounds;
            self.items.ptr[index] = item;
        }

        pub fn swapRemove(self: *@This(), index: usize) !T {
            if (index >= self.len) return error.IndexOutOfBounds;
            const old = self.items.ptr[index];
            self.len -= 1;
            if (index != self.len) {
                self.items.ptr[index] = self.items.ptr[self.len];
            }
            return old;
        }

        pub fn clear(self: *@This()) void {
            self.len = 0;
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// HashMap — Hash table with open addressing (linear probing)
// ═══════════════════════════════════════════════════════════════════════════

pub fn HashMap(comptime K: type, comptime V: type) type {
    return struct {
        entries: []Entry,
        len: usize,
        capacity: usize,
        allocator: Allocator,

        const Entry = struct {
            key: K = undefined,
            value: V = undefined,
            occupied: bool = false,
        };

        pub const empty: @This() = .{
            .entries = &.{},
            .len = 0,
            .capacity = 0,
            .allocator = undefined,
        };

        pub fn init(allocator: Allocator) @This() {
            return .{
                .entries = &.{},
                .len = 0,
                .capacity = 0,
                .allocator = allocator,
            };
        }

        pub fn deinit(self: *@This()) void {
            self.allocator.free(self.entries.ptr[0..self.capacity]);
            self.* = empty;
        }

        pub fn put(self: *@This(), key: K, value: V) !void {
            if (@as(f64, @floatFromInt(self.len)) >= @as(f64, @floatFromInt(self.capacity)) * 0.7) {
                try self.resize();
            }

            const hash = std.hash_map.getAutoHashFn(K, std.hash_map.DefaultContext(K))(key, {});
            var index = hash % self.capacity;

            while (true) : (index = (index + 1) % self.capacity) {
                if (!self.entries.ptr[index].occupied) {
                    self.entries.ptr[index] = .{
                        .key = key,
                        .value = value,
                        .occupied = true,
                    };
                    self.len += 1;
                    return;
                }
                if (std.hash_map.getAutoHashFn(K, std.hash_map.DefaultContext(K))(self.entries.ptr[index].key, {}) == hash and
                    std.meta.eql(self.entries.ptr[index].key, key))
                {
                    self.entries.ptr[index].value = value;
                    return;
                }
            }
        }

        pub fn get(self: *const @This(), key: K) ?V {
            if (self.capacity == 0) return null;
            const hash = std.hash_map.getAutoHashFn(K, std.hash_map.DefaultContext(K))(key, {});
            var index = hash % self.capacity;

            while (true) : (index = (index + 1) % self.capacity) {
                if (!self.entries.ptr[index].occupied) return null;
                if (std.hash_map.getAutoHashFn(K, std.hash_map.DefaultContext(K))(self.entries.ptr[index].key, {}) == hash and
                    std.meta.eql(self.entries.ptr[index].key, key))
                {
                    return self.entries.ptr[index].value;
                }
            }
        }

        pub fn contains(self: *const @This(), key: K) bool {
            return self.get(key) != null;
        }

        fn resize(self: *@This()) !void {
            const new_capacity = @max(self.capacity * 2, 16);
            var new_map = @This(){
                .entries = try self.allocator.alloc(Entry, new_capacity),
                .len = 0,
                .capacity = new_capacity,
                .allocator = self.allocator,
            };
            for (self.entries.ptr[0..self.capacity]) |entry| {
                if (entry.occupied) {
                    try new_map.put(entry.key, entry.value);
                }
            }
            self.allocator.free(self.entries.ptr[0..self.capacity]);
            self.entries = new_map.entries;
            self.capacity = new_map.capacity;
            self.len = new_map.len;
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// Stack — LIFO data structure
// ═══════════════════════════════════════════════════════════════════════════

pub fn Stack(comptime T: type) type {
    return struct {
        list: ArrayList(T),

        pub fn init(allocator: Allocator) @This() {
            return .{ .list = ArrayList(T).init(allocator) };
        }

        pub fn deinit(self: *@This()) void {
            self.list.deinit();
        }

        pub fn push(self: *@This(), item: T) !void {
            try self.list.append(item);
        }

        pub fn pop(self: *@This()) !T {
            if (self.list.len == 0) return error.EmptyStack;
            self.list.len -= 1;
            return self.list.items.ptr[self.list.len];
        }

        pub fn peek(self: *const @This()) ?T {
            if (self.list.len == 0) return null;
            return self.list.items.ptr[self.list.len - 1];
        }

        pub fn size(self: *const @This()) usize {
            return self.list.len;
        }

        pub fn isEmpty(self: *const @This()) bool {
            return self.list.len == 0;
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// Queue — Circular buffer FIFO
// ═══════════════════════════════════════════════════════════════════════════

pub fn Queue(comptime T: type) type {
    return struct {
        buffer: []T,
        head: usize,
        tail: usize,
        len: usize,
        capacity: usize,
        allocator: Allocator,

        pub fn init(allocator: Allocator, cap: usize) !@This() {
            return .{
                .buffer = try allocator.alloc(T, cap),
                .head = 0,
                .tail = 0,
                .len = 0,
                .capacity = cap,
                .allocator = allocator,
            };
        }

        pub fn deinit(self: *@This()) void {
            self.allocator.free(self.buffer);
        }

        pub fn enqueue(self: *@This(), item: T) !void {
            if (self.len == self.capacity) return error.QueueFull;
            self.buffer[self.tail] = item;
            self.tail = (self.tail + 1) % self.capacity;
            self.len += 1;
        }

        pub fn dequeue(self: *@This()) !T {
            if (self.len == 0) return error.EmptyQueue;
            const item = self.buffer[self.head];
            self.head = (self.head + 1) % self.capacity;
            self.len -= 1;
            return item;
        }

        pub fn peek(self: *const @This()) ?T {
            if (self.len == 0) return null;
            return self.buffer[self.head];
        }

        pub fn size(self: *const @This()) usize {
            return self.len;
        }

        pub fn isEmpty(self: *const @This()) bool {
            return self.len == 0;
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// BinarySearchTree — Ordered tree
// ═══════════════════════════════════════════════════════════════════════════

pub fn BinarySearchTree(comptime T: type) type {
    return struct {
        root: ?*Node,
        len: usize,
        allocator: Allocator,

        const Node = struct {
            data: T,
            left: ?*Node = null,
            right: ?*Node = null,
        };

        pub fn init(allocator: Allocator) @This() {
            return .{ .root = null, .len = 0, .allocator = allocator };
        }

        pub fn deinit(self: *@This()) void {
            if (self.root) |root| {
                self.deinitNode(root);
            }
        }

        fn deinitNode(self: *@This(), node: *Node) void {
            if (node.left) |left| self.deinitNode(left);
            if (node.right) |right| self.deinitNode(right);
            self.allocator.destroy(node);
        }

        pub fn insert(self: *@This(), data: T) !void {
            const node = try self.allocator.create(Node);
            node.* = .{ .data = data };
            if (self.root) |root| {
                try self.insertNode(root, node);
            } else {
                self.root = node;
            }
            self.len += 1;
        }

        fn insertNode(self: *@This(), current: *Node, new_node: *Node) !void {
            if (new_node.data < current.data) {
                if (current.left) |left| {
                    try self.insertNode(left, new_node);
                } else {
                    current.left = new_node;
                }
            } else if (new_node.data > current.data) {
                if (current.right) |right| {
                    try self.insertNode(right, new_node);
                } else {
                    current.right = new_node;
                }
            }
            // Duplicate — do nothing
        }

        pub fn contains(self: *const @This(), data: T) bool {
            var current = self.root;
            while (current) |node| {
                if (data == node.data) return true;
                current = if (data < node.data) node.left else node.right;
            }
            return false;
        }

        pub fn inorder(self: *const @This(), result: *ArrayList(T)) !void {
            if (self.root) |root| {
                try self.inorderRec(root, result);
            }
        }

        fn inorderRec(self: *const @This(), node: *Node, result: *ArrayList(T)) !void {
            if (node.left) |left| try self.inorderRec(left, result);
            try result.append(node.data);
            if (node.right) |right| try self.inorderRec(right, result);
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// MinHeap — Priority queue
// ═══════════════════════════════════════════════════════════════════════════

pub fn MinHeap(comptime T: type) type {
    return struct {
        data: ArrayList(T),

        pub fn init(allocator: Allocator) @This() {
            return .{ .data = ArrayList(T).init(allocator) };
        }

        pub fn deinit(self: *@This()) void {
            self.data.deinit();
        }

        pub fn push(self: *@This(), item: T) !void {
            try self.data.append(item);
            self.siftUp(self.data.len - 1);
        }

        pub fn pop(self: *@This()) !T {
            if (self.data.len == 0) return error.EmptyHeap;
            const min = self.data.items.ptr[0];
            self.data.len -= 1;
            if (self.data.len > 0) {
                self.data.items.ptr[0] = self.data.items.ptr[self.data.len];
                self.siftDown(0);
            }
            return min;
        }

        pub fn peek(self: *const @This()) ?T {
            if (self.data.len == 0) return null;
            return self.data.items.ptr[0];
        }

        pub fn size(self: *const @This()) usize {
            return self.data.len;
        }

        fn siftUp(self: *@This(), index: usize) void {
            var i = index;
            while (i > 0) {
                const parent = (i - 1) / 2;
                if (self.data.items.ptr[i] < self.data.items.ptr[parent]) {
                    const tmp = self.data.items.ptr[i];
                    self.data.items.ptr[i] = self.data.items.ptr[parent];
                    self.data.items.ptr[parent] = tmp;
                    i = parent;
                } else break;
            }
        }

        fn siftDown(self: *@This(), index: usize) void {
            var i = index;
            while (true) {
                const left = 2 * i + 1;
                const right = 2 * i + 2;
                var smallest = i;

                if (left < self.data.len and self.data.items.ptr[left] < self.data.items.ptr[smallest]) {
                    smallest = left;
                }
                if (right < self.data.len and self.data.items.ptr[right] < self.data.items.ptr[smallest]) {
                    smallest = right;
                }
                if (smallest != i) {
                    const tmp = self.data.items.ptr[i];
                    self.data.items.ptr[i] = self.data.items.ptr[smallest];
                    self.data.items.ptr[smallest] = tmp;
                    i = smallest;
                } else break;
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════
// Union-Find — Disjoint set
// ═══════════════════════════════════════════════════════════════════════════

pub const UnionFind = struct {
    parent: []usize,
    rank: []usize,
    components: usize,
    allocator: Allocator,

    pub fn init(allocator: Allocator, n: usize) !@This() {
        var parent = try allocator.alloc(usize, n);
        var rank = try allocator.alloc(usize, n);
        for (0..n) |i| {
            parent[i] = i;
            rank[i] = 0;
        }
        return .{ .parent = parent, .rank = rank, .components = n, .allocator = allocator };
    }

    pub fn deinit(self: *@This()) void {
        self.allocator.free(self.parent);
        self.allocator.free(self.rank);
    }

    pub fn find(self: *@This(), x: usize) usize {
        if (self.parent[x] != x) {
            self.parent[x] = self.find(self.parent[x]); // Path compression
        }
        return self.parent[x];
    }

    pub fn union(self: *@This(), x: usize, y: usize) bool {
        const root_x = self.find(x);
        const root_y = self.find(y);
        if (root_x == root_y) return false;

        if (self.rank[root_x] < self.rank[root_y]) {
            self.parent[root_x] = root_y;
        } else if (self.rank[root_x] > self.rank[root_y]) {
            self.parent[root_y] = root_x;
        } else {
            self.parent[root_y] = root_x;
            self.rank[root_x] += 1;
        }
        self.components -= 1;
        return true;
    }

    pub fn connected(self: *@This(), x: usize, y: usize) bool {
        return self.find(x) == self.find(y);
    }
};

// ═══════════════════════════════════════════════════════════════════════════
// Demo / Test
// ═══════════════════════════════════════════════════════════════════════════

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    print("=== Ruva Data Structures Library (Zig) ===\n\n", .{});

    // ArrayList test
    var list = ArrayList(i32).init(allocator);
    defer list.deinit();
    try list.append(10);
    try list.append(20);
    try list.append(30);
    print("ArrayList: {d}, {d}, {d}\n", .{ list.get(0).?, list.get(1).?, list.get(2).? });
    print("  size: {d}\n", .{list.len});

    // Stack test
    var stack = Stack(i32).init(allocator);
    defer stack.deinit();
    try stack.push(100);
    try stack.push(200);
    try stack.push(300);
    print("\nStack pop: {d}, {d}\n", .{ try stack.pop(), try stack.pop() });

    // HashMap test
    var map = HashMap([]const u8, i32).init(allocator);
    defer map.deinit();
    try map.put("one", 1);
    try map.put("two", 2);
    try map.put("three", 3);
    print("\nHashMap: one={d}, two={d}\n", .{ map.get("one").?, map.get("two").? });

    // BinarySearchTree test
    var tree = BinarySearchTree(i32).init(allocator);
    defer tree.deinit();
    for ([_]i32{ 5, 3, 7, 1, 4, 6, 8 }) |val| {
        try tree.insert(val);
    }
    var sorted = ArrayList(i32).init(allocator);
    defer sorted.deinit();
    try tree.inorder(&sorted);
    print("\nBST inorder: ", .{});
    for (sorted.itemsSlice()) |val| {
        print("{d} ", .{val});
    }
    print("\n  size: {d}\n", .{tree.len});

    // MinHeap test
    var heap = MinHeap(i32).init(allocator);
    defer heap.deinit();
    for ([_]i32{ 5, 3, 7, 1, 4, 6, 8 }) |val| {
        try heap.push(val);
    }
    print("\nMinHeap poll order: ", .{});
    while (heap.size() > 0) {
        print("{d} ", .{try heap.pop()});
    }
    print("\n", .{});

    // UnionFind test
    var uf = try UnionFind.init(allocator, 6);
    defer uf.deinit();
    _ = uf.union(0, 1);
    _ = uf.union(1, 2);
    _ = uf.union(3, 4);
    print("\nUnionFind: 0~2 connected? {}\n", .{uf.connected(0, 2)});
    print("  0~3 connected? {}\n", .{uf.connected(0, 3)});
    print("  components: {d}\n", .{uf.components});

    print("\nAll data structures functional.\n", .{});
}
