// CPU Benchmark - Java
// Tests: Fibonacci, Matrix Multiply, Sorting, Prime Sieve

public class CpuBenchmark {
    
    // ─── Fibonacci (Recursive) ──────────────────────────────────────────────
    
    static long fibRecursive(long n) {
        if (n <= 1) return n;
        return fibRecursive(n - 1) + fibRecursive(n - 2);
    }
    
    static double benchmarkFibonacci() {
        long start = System.currentTimeMillis();
        
        long sum = 0;
        for (int i = 0; i < 40; i++) {
            sum += fibRecursive(i);
        }
        
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("Fibonacci(40): " + elapsed + " ms");
        return elapsed;
    }
    
    // ─── Matrix Multiply ────────────────────────────────────────────────────
    
    static void matrixMultiply(double[][] a, double[][] b, double[][] c, int n) {
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                c[i][j] = 0.0;
                for (int k = 0; k < n; k++) {
                    c[i][j] += a[i][k] * b[k][j];
                }
            }
        }
    }
    
    static double benchmarkMatrixMultiply() {
        int n = 512;
        double[][] a = new double[n][n];
        double[][] b = new double[n][n];
        double[][] c = new double[n][n];
        
        // Initialize matrices
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                a[i][j] = i * j;
                b[i][j] = i + j;
            }
        }
        
        long start = System.currentTimeMillis();
        matrixMultiply(a, b, c, n);
        long elapsed = System.currentTimeMillis() - start;
        
        System.out.println("Matrix Multiply (512x512): " + elapsed + " ms");
        return elapsed;
    }
    
    // ─── Sorting (Quicksort) ────────────────────────────────────────────────
    
    static void quicksort(long[] arr, int low, int high) {
        if (low < high) {
            int pivot = partition(arr, low, high);
            quicksort(arr, low, pivot - 1);
            quicksort(arr, pivot + 1, high);
        }
    }
    
    static int partition(long[] arr, int low, int high) {
        long pivot = arr[high];
        int i = low - 1;
        
        for (int j = low; j < high; j++) {
            if (arr[j] < pivot) {
                i++;
                long temp = arr[i];
                arr[i] = arr[j];
                arr[j] = temp;
            }
        }
        long temp = arr[i + 1];
        arr[i + 1] = arr[high];
        arr[high] = temp;
        return i + 1;
    }
    
    static double benchmarkSorting() {
        int n = 1000000;
        long[] arr = new long[n];
        for (int i = 0; i < n; i++) {
            arr[i] = n - i - 1;
        }
        
        long start = System.currentTimeMillis();
        quicksort(arr, 0, n - 1);
        long elapsed = System.currentTimeMillis() - start;
        
        System.out.println("Sorting (1M elements): " + elapsed + " ms");
        return elapsed;
    }
    
    // ─── Prime Sieve (Sieve of Eratosthenes) ────────────────────────────────
    
    static int sieveOfEratosthenes(int limit) {
        boolean[] sieve = new boolean[limit + 1];
        java.util.Arrays.fill(sieve, true);
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
        for (boolean prime : sieve) {
            if (prime) count++;
        }
        return count;
    }
    
    static double benchmarkPrimeSieve() {
        int limit = 10000000;
        
        long start = System.currentTimeMillis();
        int count = sieveOfEratosthenes(limit);
        long elapsed = System.currentTimeMillis() - start;
        
        System.out.println("Prime Sieve (" + limit + "): " + count + " primes in " + elapsed + " ms");
        return elapsed;
    }
    
    // ─── String Processing ──────────────────────────────────────────────────
    
    static String stringConcatenation(int n) {
        StringBuilder result = new StringBuilder();
        for (int i = 0; i < n; i++) {
            result.append("Item ").append(i).append(" ");
        }
        return result.toString();
    }
    
    static double benchmarkStringProcessing() {
        int n = 100000;
        
        long start = System.currentTimeMillis();
        String result = stringConcatenation(n);
        long elapsed = System.currentTimeMillis() - start;
        
        System.out.println("String Concatenation (" + n + "): " + elapsed + " ms");
        return elapsed;
    }
    
    // ─── Main ───────────────────────────────────────────────────────────────
    
    public static void main(String[] args) {
        System.out.println("=== Java CPU Benchmark ===");
        System.out.println();
        
        double total = 0;
        
        total += benchmarkFibonacci();
        total += benchmarkMatrixMultiply();
        total += benchmarkSorting();
        total += benchmarkPrimeSieve();
        total += benchmarkStringProcessing();
        
        System.out.println();
        System.out.println("Total: " + total + " ms");
    }
}
