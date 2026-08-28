// CPU Benchmark - C++
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve

#include <iostream>
#include <vector>
#include <chrono>
#include <string>
#include <algorithm>
#include <numeric>

using namespace std;
using namespace std::chrono;

// ─── Fibonacci (Recursive) ──────────────────────────────────────────────────

unsigned long long fib_recursive(unsigned long long n) {
    if (n <= 1) return n;
    return fib_recursive(n - 1) + fib_recursive(n - 2);
}

double benchmark_fibonacci() {
    auto start = high_resolution_clock::now();
    
    unsigned long long sum = 0;
    for (int i = 0; i < 40; i++) {
        sum += fib_recursive(i);
    }
    
    auto end = high_resolution_clock::now();
    double elapsed = duration_cast<milliseconds>(end - start).count();
    cout << "Fibonacci(40): " << elapsed << " ms" << endl;
    return elapsed;
}

// ─── Matrix Multiply ────────────────────────────────────────────────────────

void matrix_multiply(const vector<vector<double>>& a, 
                     const vector<vector<double>>& b, 
                     vector<vector<double>>& c, int n) {
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            c[i][j] = 0.0;
            for (int k = 0; k < n; k++) {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
}

double benchmark_matrix_multiply() {
    int n = 512;
    vector<vector<double>> a(n, vector<double>(n));
    vector<vector<double>> b(n, vector<double>(n));
    vector<vector<double>> c(n, vector<double>(n));
    
    // Initialize matrices
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            a[i][j] = i * j;
            b[i][j] = i + j;
        }
    }
    
    auto start = high_resolution_clock::now();
    matrix_multiply(a, b, c, n);
    auto end = high_resolution_clock::now();
    double elapsed = duration_cast<milliseconds>(end - start).count();
    
    cout << "Matrix Multiply (512x512): " << elapsed << " ms" << endl;
    return elapsed;
}

// ─── Sorting (Quicksort) ────────────────────────────────────────────────────

void quicksort(vector<long long>& arr, int low, int high) {
    if (low < high) {
        int pivot = partition(arr, low, high);
        quicksort(arr, low, pivot - 1);
        quicksort(arr, pivot + 1, high);
    }
}

int partition(vector<long long>& arr, int low, int high) {
    long long pivot = arr[high];
    int i = low - 1;
    
    for (int j = low; j < high; j++) {
        if (arr[j] < pivot) {
            i++;
            swap(arr[i], arr[j]);
        }
    }
    swap(arr[i + 1], arr[high]);
    return i + 1;
}

double benchmark_sorting() {
    int n = 1000000;
    vector<long long> arr(n);
    iota(arr.rbegin(), arr.rend(), 0);
    
    auto start = high_resolution_clock::now();
    quicksort(arr, 0, n - 1);
    auto end = high_resolution_clock::now();
    double elapsed = duration_cast<milliseconds>(end - start).count();
    
    cout << "Sorting (1M elements): " << elapsed << " ms" << endl;
    return elapsed;
}

// ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────────

int sieve_of_eratosthenes(int limit) {
    vector<bool> sieve(limit + 1, true);
    sieve[0] = false;
    sieve[1] = false;
    
    for (int p = 2; p * p <= limit; p++) {
        if (sieve[p]) {
            for (int i = p * p; i <= limit; i += p) {
                sieve[i] = false;
            }
        }
    }
    
    int count = 0;
    for (bool prime : sieve) {
        if (prime) count++;
    }
    return count;
}

double benchmark_prime_sieve() {
    int limit = 10000000;
    
    auto start = high_resolution_clock::now();
    int count = sieve_of_eratosthenes(limit);
    auto end = high_resolution_clock::now();
    double elapsed = duration_cast<milliseconds>(end - start).count();
    
    cout << "Prime Sieve (" << limit << "): " << count << " primes in " << elapsed << " ms" << endl;
    return elapsed;
}

// ─── String Processing ──────────────────────────────────────────────────────

string string_concatenation(int n) {
    string result;
    for (int i = 0; i < n; i++) {
        result += "Item " + to_string(i) + " ";
    }
    return result;
}

double benchmark_string_processing() {
    int n = 100000;
    
    auto start = high_resolution_clock::now();
    string result = string_concatenation(n);
    auto end = high_resolution_clock::now();
    double elapsed = duration_cast<milliseconds>(end - start).count();
    
    cout << "String Concatenation (" << n << "): " << elapsed << " ms" << endl;
    return elapsed;
}

// ─── Main ───────────────────────────────────────────────────────────────────

int main() {
    cout << "=== C++ CPU Benchmark ===" << endl;
    cout << endl;
    
    double total = 0;
    
    total += benchmark_fibonacci();
    total += benchmark_matrix_multiply();
    total += benchmark_sorting();
    total += benchmark_prime_sieve();
    total += benchmark_string_processing();
    
    cout << endl;
    cout << "Total: " << total << " ms" << endl;
    
    return 0;
}
