#!/usr/bin/env python3
"""
Ruva Algorithms Library — Python Reference Implementations

Common algorithms for benchmarking and reference:
  - Sorting: QuickSort, MergeSort, HeapSort, RadixSort, TimSort
  - Searching: Binary Search, KMP String Matching
  - Graph: BFS, DFS, Dijkstra, Topological Sort
  - Math: GCD, Fast Exponentiation, Modular Arithmetic, Sieve
  - Dynamic Programming: Fibonacci, LCS, Knapsack
"""

from __future__ import annotations
import time
import math
from collections import deque
from typing import Any, Callable, List, Optional, Tuple


# ═══════════════════════════════════════════════════════════════════════════
# Sorting Algorithms
# ═══════════════════════════════════════════════════════════════════════════

def quick_sort(arr: list[int]) -> list[int]:
    """QuickSort — O(n log n) average."""
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quick_sort(left) + middle + quick_sort(right)


def merge_sort(arr: list[int]) -> list[int]:
    """MergeSort — O(n log n) guaranteed."""
    if len(arr) <= 1:
        return arr
    mid = len(arr) // 2
    left = merge_sort(arr[:mid])
    right = merge_sort(arr[mid:])
    return _merge(left, right)


def _merge(left: list[int], right: list[int]) -> list[int]:
    result: list[int] = []
    i = j = 0
    while i < len(left) and j < len(right):
        if left[i] <= right[j]:
            result.append(left[i])
            i += 1
        else:
            result.append(right[j])
            j += 1
    result.extend(left[i:])
    result.extend(right[j:])
    return result


def heap_sort(arr: list[int]) -> list[int]:
    """HeapSort — O(n log n) using heapq."""
    import heapq
    heap = list(arr)
    heapq.heapify(heap)
    return [heapq.heappop(heap) for _ in range(len(heap))]


def radix_sort(arr: list[int]) -> list[int]:
    """RadixSort — O(d × (n + b)) using counting sort by digit."""
    if not arr:
        return arr

    max_val = max(abs(x) for x in arr)
    exp = 1
    result = list(arr)

    while max_val // exp > 0:
        result = _counting_sort_by_digit(result, exp)
        exp *= 10

    return result


def _counting_sort_by_digit(arr: list[int], exp: int) -> list[int]:
    n = len(arr)
    output = [0] * n
    count = [0] * 10

    for val in arr:
        digit = abs(val) // exp % 10
        count[digit] += 1

    for i in range(1, 10):
        count[i] += count[i - 1]

    for i in range(n - 1, -1, -1):
        digit = abs(arr[i]) // exp % 10
        count[digit] -= 1
        output[count[digit]] = arr[i]

    return output


def tim_sort(arr: list[int]) -> list[int]:
    """TimSort — Python's built-in Timsort wrapper."""
    return sorted(arr)


# ═══════════════════════════════════════════════════════════════════════════
# Searching Algorithms
# ═══════════════════════════════════════════════════════════════════════════

def binary_search(arr: list[int], target: int) -> int:
    """Binary Search — O(log n) on sorted array. Returns index or -1."""
    lo, hi = 0, len(arr) - 1
    while lo <= hi:
        mid = lo + (hi - lo) // 2
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            lo = mid + 1
        else:
            hi = mid - 1
    return -1


def kmp_search(text: str, pattern: str) -> int:
    """KMP String Search — O(n + m). Returns index or -1."""
    if not pattern:
        return 0
    lps = _compute_lps(pattern)
    i = j = 0

    while i < len(text):
        if text[i] == pattern[j]:
            i += 1
            j += 1
            if j == len(pattern):
                return i - j
        elif j > 0:
            j = lps[j - 1]
        else:
            i += 1

    return -1


def _compute_lps(pattern: str) -> list[int]:
    lps = [0] * len(pattern)
    length = 0
    i = 1
    while i < len(pattern):
        if pattern[i] == pattern[length]:
            length += 1
            lps[i] = length
            i += 1
        elif length > 0:
            length = lps[length - 1]
        else:
            lps[i] = 0
            i += 1
    return lps


# ═══════════════════════════════════════════════════════════════════════════
# Graph Algorithms
# ═══════════════════════════════════════════════════════════════════════════

def bfs(adj: list[list[int]], start: int) -> list[int]:
    """BFS — Breadth-First Search."""
    visited = [False] * len(adj)
    queue: deque[int] = deque([start])
    visited[start] = True
    result: list[int] = []

    while queue:
        node = queue.popleft()
        result.append(node)
        for neighbor in adj[node]:
            if not visited[neighbor]:
                visited[neighbor] = True
                queue.append(neighbor)

    return result


def dfs(adj: list[list[int]], start: int) -> list[int]:
    """DFS — Depth-First Search (iterative)."""
    visited = [False] * len(adj)
    stack = [start]
    result: list[int] = []

    while stack:
        node = stack.pop()
        if visited[node]:
            continue
        visited[node] = True
        result.append(node)
        for neighbor in reversed(adj[node]):
            if not visited[neighbor]:
                stack.append(neighbor)

    return result


def dijkstra(adj: list[list[Tuple[int, int]]], src: int) -> list[int]:
    """Dijkstra's Shortest Path — O((V + E) log V)."""
    import heapq
    n = len(adj)
    dist = [float("inf")] * n
    dist[src] = 0
    pq: list[Tuple[int, int]] = [(0, src)]

    while pq:
        d, u = heapq.heappop(pq)
        if d > dist[u]:
            continue
        for v, w in adj[u]:
            if dist[u] + w < dist[v]:
                dist[v] = dist[u] + w
                heapq.heappush(pq, (dist[v], v))

    return dist


def topological_sort(adj: list[list[int]]) -> list[int]:
    """Topological Sort — Kahn's algorithm."""
    n = len(adj)
    in_degree = [0] * n
    for u in range(n):
        for v in adj[u]:
            in_degree[v] += 1

    queue: deque[int] = deque()
    for i in range(n):
        if in_degree[i] == 0:
            queue.append(i)

    result: list[int] = []
    while queue:
        u = queue.popleft()
        result.append(u)
        for v in adj[u]:
            in_degree[v] -= 1
            if in_degree[v] == 0:
                queue.append(v)

    if len(result) != n:
        raise ValueError("Graph has a cycle")
    return result


# ═══════════════════════════════════════════════════════════════════════════
# Math Utilities
# ═══════════════════════════════════════════════════════════════════════════

def gcd(a: int, b: int) -> int:
    """Greatest Common Divisor."""
    while b:
        a, b = b, a % b
    return a


def mod_pow(base: int, exp: int, mod: int) -> int:
    """Fast modular exponentiation — O(log n)."""
    result = 1
    base %= mod
    while exp > 0:
        if exp & 1:
            result = result * base % mod
        exp >>= 1
        base = base * base % mod
    return result


def extended_gcd(a: int, b: int) -> Tuple[int, int, int]:
    """Extended Euclidean Algorithm — returns (gcd, x, y) such that ax + by = gcd."""
    if b == 0:
        return a, 1, 0
    gcd_val, x, y = extended_gcd(b, a % b)
    return gcd_val, y, x - (a // b) * y


def sieve_of_eratosthenes(limit: int) -> list[int]:
    """Sieve of Eratosthenes — returns list of primes up to limit."""
    sieve = [True] * (limit + 1)
    sieve[0] = sieve[1] = False

    p = 2
    while p * p <= limit:
        if sieve[p]:
            for i in range(p * p, limit + 1, p):
                sieve[i] = False
        p += 1

    return [i for i, is_prime in enumerate(sieve) if is_prime]


# ═══════════════════════════════════════════════════════════════════════════
# Dynamic Programming
# ═══════════════════════════════════════════════════════════════════════════

def fibonacci_dp(n: int) -> int:
    """Fibonacci with bottom-up DP — O(n) time, O(1) space."""
    if n <= 1:
        return n
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b


def longest_common_subsequence(s1: str, s2: str) -> int:
    """LCS — O(mn) time, O(mn) space."""
    m, n = len(s1), len(s2)
    dp = [[0] * (n + 1) for _ in range(m + 1)]

    for i in range(1, m + 1):
        for j in range(1, n + 1):
            if s1[i - 1] == s2[j - 1]:
                dp[i][j] = dp[i - 1][j - 1] + 1
            else:
                dp[i][j] = max(dp[i - 1][j], dp[i][j - 1])

    return dp[m][n]


def knapsack_01(weights: list[int], values: list[int], capacity: int) -> int:
    """0/1 Knapsack — O(n × W) time."""
    n = len(weights)
    dp = [0] * (capacity + 1)

    for i in range(n):
        for w in range(capacity, weights[i] - 1, -1):
            dp[w] = max(dp[w], dp[w - weights[i]] + values[i])

    return dp[capacity]


# ═══════════════════════════════════════════════════════════════════════════
# Demo / Test
# ═══════════════════════════════════════════════════════════════════════════

def main() -> None:
    print("=== Ruva Algorithms Library (Python) ===\n")

    data = [38, 27, 43, 3, 9, 82, 10]

    # Sorting
    print(f"QuickSort:    {quick_sort(data)}")
    print(f"MergeSort:    {merge_sort(data)}")
    print(f"HeapSort:     {heap_sort(data)}")
    print(f"RadixSort:    {radix_sort(data)}")
    print(f"TimSort:      {tim_sort(data)}")

    # Searching
    sorted_arr = [1, 3, 5, 7, 9, 11, 13]
    print(f"\nBinarySearch(7): index={binary_search(sorted_arr, 7)}")
    print(f"KMP('ABABD', 'ABD'): index={kmp_search('ABABDABACDABABCABAB', 'ABD')}")

    # Graph
    graph: list[list[int]] = [[] for _ in range(5)]
    graph[0].extend([1, 2])
    graph[1].extend([3, 4])
    graph[2].append(4)

    print(f"\nBFS from 0:  {bfs(graph, 0)}")
    print(f"DFS from 0:  {dfs(graph, 0)}")

    # Math
    print(f"\ngcd(48, 18):  {gcd(48, 18)}")
    print(f"modPow(2,10,1000): {mod_pow(2, 10, 1000)}")
    print(f"Primes <= 50: {len(sieve_of_eratosthenes(50))} primes")

    # DP
    print(f"\nfibonacci(40): {fibonacci_dp(40)}")
    print(f"LCS('ABCBDAB', 'BDCAB'): {longest_common_subsequence('ABCBDAB', 'BDCAB')}")
    print(f"Knapsack(4 items, W=7): {knapsack_01([1, 3, 4, 5], [1, 4, 5, 7], 7)}")

    print("\nAll algorithms functional.")


if __name__ == "__main__":
    main()
