// CPU Benchmark - C
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdint.h>

// ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

uint64_t fib_recursive(uint64_t n) {
    if (n <= 1) return n;
    return fib_recursive(n - 1) + fib_recursive(n - 2);
}

double benchmark_fibonacci() {
    clock_t start = clock();
    
    uint64_t sum = 0;
    for (int i = 0; i < 40; i++) {
        sum += fib_recursive(i);
    }
    
    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    printf("Fibonacci(40): %.2f ms\n", elapsed);
    return elapsed;
}

// ─── Matrix Multiply ────────────────────────────────────────────────────────

void matrix_multiply(double** a, double** b, double** c, int n) {
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            c[i][j] = 0.0;
            for (int k = 0; k < n; k++) {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

double** create_matrix(int n) {
    double** mat = (double**)malloc(n * sizeof(double*));
    for (int i = 0; i < n; i++) {
        mat[i] = (double*)malloc(n * sizeof(double));
    }
    return mat;
}

void free_matrix(double** mat, int n) {
    for (int i = 0; i < n; i++) {
        free(mat[i]);
    }
    free(mat);
}

double benchmark_matrix_multiply() {
    int n = 512;
    double** a = create_matrix(n);
    double** b = create_matrix(n);
    double** c = create_matrix(n);
    
    // Initialize matrices
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            a[i][j] = i * j;
            b[i][j] = i + j;
        }
    }
    
    clock_t start = clock();
    matrix_multiply(a, b, c, n);
    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    
    printf("Matrix Multiply (512x512): %.2f ms\n", elapsed);
    
    free_matrix(a, n);
    free_matrix(b, n);
    free_matrix(c, n);
    
    return elapsed;
}

// ─── Sorting (Quicksort) ────────────────────────────────────────────────────

void quicksort(int64_t* arr, int low, int high) {
    if (low < high) {
        int pivot = partition(arr, low, high);
        quicksort(arr, low, pivot - 1);
        quicksort(arr, pivot + 1, high);
    }
}

int partition(int64_t* arr, int low, int high) {
    int64_t pivot = arr[high];
    int i = low - 1;
    
    for (int j = low; j < high; j++) {
        if (arr[j] < pivot) {
            i++;
            int64_t temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
        }
    }
    int64_t temp = arr[i + 1];
    arr[i + 1] = arr[high];
    arr[high] = temp;
    return i + 1;
}

double benchmark_sorting() {
    int n = 1000000;
    int64_t* arr = (int64_t*)malloc(n * sizeof(int64_t));
    
    for (int i = 0; i < n; i++) {
        arr[i] = n - i - 1;
    }
    
    clock_t start = clock();
    quicksort(arr, 0, n - 1);
    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    
    printf("Sorting (1M elements): %.2f ms\n", elapsed);
    
    free(arr);
    return elapsed;
}

// ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────────

int sieve_of_eratosthenes(int limit) {
    char* sieve = (char*)malloc((limit + 1) * sizeof(char));
    memset(sieve, 1, limit + 1);
    sieve[0] = 0;
    sieve[1] = 0;
    
    for (int p = 2; p * p <= limit; p++) {
        if (sieve[p]) {
            for (int i = p * p; i <= limit; i += p) {
                sieve[i] = 0;
            }
        }
    }
    
    int count = 0;
    for (int i = 0; i <= limit; i++) {
        if (sieve[i]) count++;
    }
    
    free(sieve);
    return count;
}

double benchmark_prime_sieve() {
    int limit = 10000000;
    
    clock_t start = clock();
    int count = sieve_of_eratosthenes(limit);
    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    
    printf("Prime Sieve (%d): %d primes in %.2f ms\n", limit, count, elapsed);
    return elapsed;
}

// ─── String Processing ──────────────────────────────────────────────────────

char* string_concatenation(int n) {
    int capacity = n * 20;
    char* result = (char*)malloc(capacity * sizeof(char));
    result[0] = '\0';
    
    for (int i = 0; i < n; i++) {
        char buffer[32];
        sprintf(buffer, "Item %d ", i);
        strcat(result, buffer);
    }
    
    return result;
}

double benchmark_string_processing() {
    int n = 100000;
    
    clock_t start = clock();
    char* result = string_concatenation(n);
    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    
    printf("String Concatenation (%d): %.2f ms\n", n, elapsed);
    
    free(result);
    return elapsed;
}

// ─── Main ───────────────────────────────────────────────────────────────────

int main() {
    printf("=== C CPU Benchmark ===\n\n");
    
    double total = 0;
    
    total += benchmark_fibonacci();
    total += benchmark_matrix_multiply();
    total += benchmark_sorting();
    total += benchmark_prime_sieve();
    total += benchmark_string_processing();
    
    printf("\nTotal: %.2f ms\n", total);
    
    return 0;
}
