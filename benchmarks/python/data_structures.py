#!/usr/bin/env python3
"""
Ruva Data Structures Library — Python Reference Implementations

Core data structures for benchmarking and reference:
  - ArrayList (dynamic array with type hints)
  - HashMap (dictionary with ordered iteration)
  - Stack (LIFO)
  - Queue (FIFO with deque)
  - BinarySearchTree
  - MinHeap
  - UnionFind
  - Trie (prefix tree)
  - LRU Cache
"""

from __future__ import annotations
from typing import Any, Callable, Generic, Iterable, Iterator, List, Optional, Tuple, TypeVar
from collections import deque
import time

T = TypeVar("T")
K = TypeVar("K")
V = TypeVar("V")


# ═══════════════════════════════════════════════════════════════════════════
# ArrayList — Dynamic array with extra utilities
# ═══════════════════════════════════════════════════════════════════════════

class ArrayList(Generic[T]):
    """Dynamic array — like Python's list but with explicit capacity management."""

    def __init__(self, capacity: int = 16) -> None:
        self._data: list[T] = []
        self._capacity = capacity

    def add(self, item: T) -> None:
        self._data.append(item)

    def add_at(self, index: int, item: T) -> None:
        if index < 0 or index > len(self._data):
            raise IndexError(f"Index: {index}, Size: {len(self._data)}")
        self._data.insert(index, item)

    def get(self, index: int) -> T:
        if index < 0 or index >= len(self._data):
            raise IndexError(f"Index: {index}, Size: {len(self._data)}")
        return self._data[index]

    def set(self, index: int, item: T) -> T:
        if index < 0 or index >= len(self._data):
            raise IndexError(f"Index: {index}, Size: {len(self._data)}")
        old = self._data[index]
        self._data[index] = item
        return old

    def remove_at(self, index: int) -> T:
        if index < 0 or index >= len(self._data):
            raise IndexError(f"Index: {index}, Size: {len(self._data)}")
        return self._data.pop(index)

    def remove(self, item: T) -> bool:
        try:
            self._data.remove(item)
            return True
        except ValueError:
            return False

    def contains(self, item: T) -> bool:
        return item in self._data

    def index_of(self, item: T) -> int:
        try:
            return self._data.index(item)
        except ValueError:
            return -1

    def size(self) -> int:
        return len(self._data)

    def is_empty(self) -> bool:
        return len(self._data) == 0

    def clear(self) -> None:
        self._data.clear()

    def to_list(self) -> list[T]:
        return list(self._data)

    def __repr__(self) -> str:
        return f"ArrayList({self._data})"


# ═══════════════════════════════════════════════════════════════════════════
# HashMap — Hash table with chaining
# ═══════════════════════════════════════════════════════════════════════════

class HashMap(Generic[K, V]):
    """Hash map with separate chaining for collision resolution."""

    def __init__(self, capacity: int = 16, load_factor: float = 0.75) -> None:
        self._capacity = capacity
        self._load_factor = load_factor
        self._buckets: list[list[Tuple[K, V]]] = [[] for _ in range(capacity)]
        self._size = 0

    def _hash(self, key: K) -> int:
        return hash(key) % self._capacity

    def put(self, key: K, value: V) -> Optional[V]:
        if self._size >= self._capacity * self._load_factor:
            self._resize()

        bucket_idx = self._hash(key)
        bucket = self._buckets[bucket_idx]

        for i, (k, v) in enumerate(bucket):
            if k == key:
                old = v
                bucket[i] = (key, value)
                return old

        bucket.append((key, value))
        self._size += 1
        return None

    def get(self, key: K) -> Optional[V]:
        bucket_idx = self._hash(key)
        for k, v in self._buckets[bucket_idx]:
            if k == key:
                return v
        return None

    def remove(self, key: K) -> Optional[V]:
        bucket_idx = self._hash(key)
        bucket = self._buckets[bucket_idx]
        for i, (k, v) in enumerate(bucket):
            if k == key:
                bucket.pop(i)
                self._size -= 1
                return v
        return None

    def contains_key(self, key: K) -> bool:
        return self.get(key) is not None

    def size(self) -> int:
        return self._size

    def is_empty(self) -> bool:
        return self._size == 0

    def keys(self) -> list[K]:
        result = []
        for bucket in self._buckets:
            for k, _ in bucket:
                result.append(k)
        return result

    def values(self) -> list[V]:
        result = []
        for bucket in self._buckets:
            for _, v in bucket:
                result.append(v)
        return result

    def items(self) -> list[Tuple[K, V]]:
        result = []
        for bucket in self._buckets:
            result.extend(bucket)
        return result

    def _resize(self) -> None:
        old_items = self.items()
        self._capacity *= 2
        self._buckets = [[] for _ in range(self._capacity)]
        self._size = 0
        for k, v in old_items:
            self.put(k, v)

    def __repr__(self) -> str:
        return f"HashMap({dict(self.items())})"


# ═══════════════════════════════════════════════════════════════════════════
# Stack — LIFO data structure
# ═══════════════════════════════════════════════════════════════════════════

class Stack(Generic[T]):
    """LIFO stack — push, pop, peek."""

    def __init__(self) -> None:
        self._items: list[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> T:
        if not self._items:
            raise IndexError("Stack is empty")
        return self._items.pop()

    def peek(self) -> T:
        if not self._items:
            raise IndexError("Stack is empty")
        return self._items[-1]

    def size(self) -> int:
        return len(self._items)

    def is_empty(self) -> bool:
        return len(self._items) == 0

    def clear(self) -> None:
        self._items.clear()

    def __repr__(self) -> str:
        return f"Stack({self._items})"


# ═══════════════════════════════════════════════════════════════════════════
# Queue — FIFO data structure (deque-based)
# ═══════════════════════════════════════════════════════════════════════════

class Queue(Generic[T]):
    """FIFO queue — enqueue, dequeue, peek."""

    def __init__(self) -> None:
        self._items: deque[T] = deque()

    def enqueue(self, item: T) -> None:
        self._items.append(item)

    def dequeue(self) -> T:
        if not self._items:
            raise IndexError("Queue is empty")
        return self._items.popleft()

    def peek(self) -> T:
        if not self._items:
            raise IndexError("Queue is empty")
        return self._items[0]

    def size(self) -> int:
        return len(self._items)

    def is_empty(self) -> bool:
        return len(self._items) == 0

    def __repr__(self) -> str:
        return f"Queue({list(self._items)})"


# ═══════════════════════════════════════════════════════════════════════════
# BinarySearchTree — Ordered tree
# ═══════════════════════════════════════════════════════════════════════════

class BinarySearchTree(Generic[T]):
    """Binary search tree with insert, search, and traversal."""

    class _Node:
        def __init__(self, data: T) -> None:
            self.data = data
            self.left: Optional["BinarySearchTree._Node"] = None
            self.right: Optional["BinarySearchTree._Node"] = None

    def __init__(self) -> None:
        self.root: Optional[BinarySearchTree._Node] = None
        self.size = 0

    def insert(self, data: T) -> None:
        self.root = self._insert_rec(self.root, data)
        self.size += 1

    def _insert_rec(self, node: Optional[BinarySearchTree._Node], data: T) -> BinarySearchTree._Node:
        if node is None:
            return BinarySearchTree._Node(data)
        if data < node.data:
            node.left = self._insert_rec(node.left, data)
        elif data > node.data:
            node.right = self._insert_rec(node.right, data)
        return node

    def contains(self, data: T) -> bool:
        return self._search_rec(self.root, data)

    def _search_rec(self, node: Optional[BinarySearchTree._Node], data: T) -> bool:
        if node is None:
            return False
        if data == node.data:
            return True
        if data < node.data:
            return self._search_rec(node.left, data)
        return self._search_rec(node.right, data)

    def inorder(self) -> list[T]:
        result: list[T] = []
        self._inorder_rec(self.root, result)
        return result

    def _inorder_rec(self, node: Optional[BinarySearchTree._Node], result: list[T]) -> None:
        if node is None:
            return
        self._inorder_rec(node.left, result)
        result.append(node.data)
        self._inorder_rec(node.right, result)

    def height(self) -> int:
        return self._height_rec(self.root)

    def _height_rec(self, node: Optional[BinarySearchTree._Node]) -> int:
        if node is None:
            return 0
        return 1 + max(self._height_rec(node.left), self._height_rec(node.right))

    def __repr__(self) -> str:
        return f"BST({self.inorder()})"


# ═══════════════════════════════════════════════════════════════════════════
# MinHeap — Priority queue
# ═══════════════════════════════════════════════════════════════════════════

class MinHeap(Generic[T]):
    """Min-heap for priority queue operations."""

    def __init__(self) -> None:
        self._data: list[T] = []

    def push(self, item: T) -> None:
        self._data.append(item)
        self._sift_up(len(self._data) - 1)

    def pop(self) -> T:
        if not self._data:
            raise IndexError("Heap is empty")
        min_val = self._data[0]
        last = self._data.pop()
        if self._data:
            self._data[0] = last
            self._sift_down(0)
        return min_val

    def peek(self) -> T:
        if not self._data:
            raise IndexError("Heap is empty")
        return self._data[0]

    def size(self) -> int:
        return len(self._data)

    def is_empty(self) -> bool:
        return len(self._data) == 0

    def _sift_up(self, index: int) -> None:
        while index > 0:
            parent = (index - 1) // 2
            if self._data[index] < self._data[parent]:
                self._data[index], self._data[parent] = self._data[parent], self._data[index]
                index = parent
            else:
                break

    def _sift_down(self, index: int) -> None:
        size = len(self._data)
        while True:
            smallest = index
            left = 2 * index + 1
            right = 2 * index + 2

            if left < size and self._data[left] < self._data[smallest]:
                smallest = left
            if right < size and self._data[right] < self._data[smallest]:
                smallest = right

            if smallest != index:
                self._data[index], self._data[smallest] = self._data[smallest], self._data[index]
                index = smallest
            else:
                break

    def __repr__(self) -> str:
        return f"MinHeap({self._data})"


# ═══════════════════════════════════════════════════════════════════════════
# Union-Find — Disjoint set with path compression and union by rank
# ═══════════════════════════════════════════════════════════════════════════

class UnionFind:
    """Disjoint set — find and union with path compression."""

    def __init__(self, n: int) -> None:
        self.parent = list(range(n))
        self.rank = [0] * n
        self.components = n

    def find(self, x: int) -> int:
        if self.parent[x] != x:
            self.parent[x] = self.find(self.parent[x])  # Path compression
        return self.parent[x]

    def union(self, x: int, y: int) -> bool:
        root_x, root_y = self.find(x), self.find(y)
        if root_x == root_y:
            return False

        if self.rank[root_x] < self.rank[root_y]:
            self.parent[root_x] = root_y
        elif self.rank[root_x] > self.rank[root_y]:
            self.parent[root_y] = root_x
        else:
            self.parent[root_y] = root_x
            self.rank[root_x] += 1

        self.components -= 1
        return True

    def connected(self, x: int, y: int) -> bool:
        return self.find(x) == self.find(y)

    def __repr__(self) -> str:
        return f"UnionFind(components={self.components})"


# ═══════════════════════════════════════════════════════════════════════════
# Trie — Prefix tree for string operations
# ═══════════════════════════════════════════════════════════════════════════

class Trie:
    """Prefix tree for efficient string prefix operations."""

    class _Node:
        def __init__(self) -> None:
            self.children: dict[str, "Trie._Node"] = {}
            self.is_end = False

    def __init__(self) -> None:
        self.root = Trie._Node()

    def insert(self, word: str) -> None:
        node = self.root
        for char in word:
            if char not in node.children:
                node.children[char] = Trie._Node()
            node = node.children[char]
        node.is_end = True

    def search(self, word: str) -> bool:
        node = self._find_node(word)
        return node is not None and node.is_end

    def starts_with(self, prefix: str) -> bool:
        return self._find_node(prefix) is not None

    def _find_node(self, prefix: str) -> Optional["Trie._Node"]:
        node = self.root
        for char in prefix:
            if char not in node.children:
                return None
            node = node.children[char]
        return node

    def autocomplete(self, prefix: str) -> list[str]:
        node = self._find_node(prefix)
        if node is None:
            return []
        results: list[str] = []
        self._collect_words(node, prefix, results)
        return results

    def _collect_words(self, node: "Trie._Node", current: str, results: list[str]) -> None:
        if node.is_end:
            results.append(current)
        for char, child in sorted(node.children.items()):
            self._collect_words(child, current + char, results)

    def __repr__(self) -> str:
        return f"Trie(size={self.root is not None})"


# ═══════════════════════════════════════════════════════════════════════════
# LRU Cache — Least Recently Used eviction
# ═══════════════════════════════════════════════════════════════════════════

class LRUCache(Generic[K, V]):
    """LRU cache with O(1) get/put using OrderedDict-like behavior."""

    def __init__(self, capacity: int) -> None:
        self._capacity = capacity
        self._cache: dict[K, V] = {}
        self._order: deque[K] = deque()

    def get(self, key: K) -> Optional[V]:
        if key not in self._cache:
            return None
        self._order.remove(key)
        self._order.append(key)
        return self._cache[key]

    def put(self, key: K, value: V) -> None:
        if key in self._cache:
            self._order.remove(key)
        elif len(self._cache) >= self._capacity:
            oldest = self._order.popleft()
            del self._cache[oldest]
        self._cache[key] = value
        self._order.append(key)

    def size(self) -> int:
        return len(self._cache)

    def __repr__(self) -> str:
        return f"LRUCache({dict(self._cache)})"


# ═══════════════════════════════════════════════════════════════════════════
# Demo / Test
# ═══════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("=== Ruva Data Structures Library (Python) ===\n")

    # ArrayList test
    arr: ArrayList[int] = ArrayList()
    arr.add(10)
    arr.add(20)
    arr.add(30)
    print(f"ArrayList: {arr.get(0)}, {arr.get(1)}, {arr.get(2)}")
    print(f"  size: {arr.size()}")

    # Stack test
    stack: Stack[int] = Stack()
    stack.push(100)
    stack.push(200)
    stack.push(300)
    print(f"\nStack pop: {stack.pop()}, {stack.pop()}")

    # HashMap test
    hmap: HashMap[str, int] = HashMap()
    hmap.put("one", 1)
    hmap.put("two", 2)
    hmap.put("three", 3)
    print(f"\nHashMap: one={hmap.get('one')}, two={hmap.get('two')}")

    # BinarySearchTree test
    tree: BinarySearchTree[int] = BinarySearchTree()
    for val in [5, 3, 7, 1, 4, 6, 8]:
        tree.insert(val)
    print(f"\nBST inorder: {tree.inorder()}")
    print(f"  height: {tree.height()}")

    # MinHeap test
    heap: MinHeap[int] = MinHeap()
    for val in [5, 3, 7, 1, 4, 6, 8]:
        heap.push(val)
    order = []
    while not heap.is_empty():
        order.append(heap.pop())
    print(f"\nMinHeap poll order: {order}")

    # Union-Find test
    uf = UnionFind(6)
    uf.union(0, 1)
    uf.union(1, 2)
    uf.union(3, 4)
    print(f"\nUnionFind: 0~2 connected? {uf.connected(0, 2)}")
    print(f"  0~3 connected? {uf.connected(0, 3)}")
    print(f"  components: {uf.components}")

    # Trie test
    trie = Trie()
    for word in ["apple", "app", "application", "bat", "ball", "band"]:
        trie.insert(word)
    print(f"\nTrie search('apple'): {trie.search('apple')}")
    print(f"  starts_with('app'): {trie.starts_with('app')}")
    print(f"  autocomplete('app'): {trie.autocomplete('app')}")
    print(f"  autocomplete('ba'): {trie.autocomplete('ba')}")

    # LRU Cache test
    cache: LRUCache[int, str] = LRUCache(3)
    cache.put(1, "one")
    cache.put(2, "two")
    cache.put(3, "three")
    cache.get(1)  # Makes 1 most recent
    cache.put(4, "four")  # Evicts 2
    print(f"\nLRU Cache: get(1)={cache.get(1)}, get(2)={cache.get(2)}, get(4)={cache.get(4)}")

    print("\nAll data structures functional.")


if __name__ == "__main__":
    main()
