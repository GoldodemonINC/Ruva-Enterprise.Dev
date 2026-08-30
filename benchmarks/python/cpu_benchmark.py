#!/usr/bin/env python3
"""
CPU Benchmark - Python
Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve
"""

import time
import math


# ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

def fib_recursive(n):
    if n <= 1:
        return n
    return fib_recursive(n - 1) + fib_recursive(n - 2)


def benchmark_fibonacci():
    start = time.time()
    total = 0
    for i in range(40):
        total += fib_recursive(i)
    elapsed = (time.time() - start) * 1000
    print(f"Fibonacci(40): {elapsed:.2f} ms")
    return elapsed


# ─── Matrix Multiply ────────────────────────────────────────────────────────

def matrix_multiply(a, b, c, n):
    for i in range(n):
        for j in range(n):
            c[i][j] = 0.0
            for k in range(n):
                c[i][j] += a[i][k] * b[k][j]


def benchmark_matrix_multiply():
    n = 512
    a = [[float(i * j) for j in range(n)] for i in range(n)]
    b = [[float(i + j) for j in range(n)] for i in range(n)]
    c = [[0.0] * n for _ in range(n)]

    start = time.time()
    matrix_multiply(a, b, c, n)
    elapsed = (time.time() - start) * 1000
    print(f"Matrix Multiply (512x512): {elapsed:.2f} ms")
    return elapsed


# ─── Sorting (Quicksort) ────────────────────────────────────────────────────

def quicksort(arr, low, high):
    if low < high:
        pi = partition(arr, low, high)
        quicksort(arr, low, pi - 1)
        quicksort(arr, pi + 1, high)


def partition(arr, low, high):
    pivot = arr[high]
    i = low - 1
    for j in range(low, high):
        if arr[j] < pivot:
            i += 1
            arr[i], arr[j] = arr[j], arr[i]
    arr[i + 1], arr[high] = arr[high], arr[i + 1]
    return i + 1


def benchmark_sorting():
    n = 1_000_000
    arr = list(range(n, 0, -1))

    start = time.time()
    quicksort(arr, 0, n - 1)
    elapsed = (time.time() - start) * 1000
    print(f"Sorting (1M elements): {elapsed:.2f} ms")
    return elapsed


# ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────────

def sieve_of_eratosthenes(limit):
    sieve = [True] * (limit + 1)
    sieve[0] = False
    sieve[1] = False

    p = 2
    while p * p <= limit:
        if sieve[p]:
            for i in range(p * p, limit + 1, p):
                sieve[i] = False
        p += 1

    return sum(sieve)


def benchmark_prime_sieve():
    limit = 10_000_000

    start = time.time()
    count = sieve_of_eratosthenes(limit)
    elapsed = (time.time() - start) * 1000
    print(f"Prime Sieve ({limit}): {count} primes in {elapsed:.2f} ms")
    return elapsed


# ─── String Processing ──────────────────────────────────────────────────────

def string_concatenation(n):
    parts = []
    for i in range(n):
        parts.append(f"Item {i} ")
    return "".join(parts)


def benchmark_string_processing():
    n = 100_000

    start = time.time()
    _result = string_concatenation(n)
    elapsed = (time.time() - start) * 1000
    print(f"String Concatenation ({n}): {elapsed:.2f} ms")
    return elapsed


# ─── Main ───────────────────────────────────────────────────────────────────

def main():
    print("=== Python CPU Benchmark ===")
    print()

    total = 0.0

    total += benchmark_fibonacci()
    total += benchmark_matrix_multiply()
    total += benchmark_sorting()
    total += benchmark_prime_sieve()
    total += benchmark_string_processing()

    print()
    print(f"Total: {total:.2f} ms")


if __name__ == "__main__":
    main()
