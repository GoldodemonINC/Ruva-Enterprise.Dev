// CPU Benchmark - Fixed (iterative quicksort)
public class CpuBenchmarkFixed {

    static long fibRecursive(long n) {
        if (n <= 1) return n;
        return fibRecursive(n - 1) + fibRecursive(n - 2);
    }

    static double benchmarkFibonacci() {
        long start = System.currentTimeMillis();
        long sum = 0;
        for (int i = 0; i < 40; i++) sum += fibRecursive(i);
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("Fibonacci(40): " + elapsed + " ms");
        return elapsed;
    }

    static double benchmarkMatrixMultiply() {
        int n = 512;
        double[][] a = new double[n][n], b = new double[n][n], c = new double[n][n];
        for (int i = 0; i < n; i++) for (int j = 0; j < n; j++) { a[i][j] = i * j; b[i][j] = i + j; }
        long start = System.currentTimeMillis();
        for (int i = 0; i < n; i++) for (int j = 0; j < n; j++) { double s = 0; for (int k = 0; k < n; k++) s += a[i][k] * b[k][j]; c[i][j] = s; }
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("Matrix Multiply (512x512): " + elapsed + " ms");
        return elapsed;
    }

    static int partition(long[] arr, int low, int high) {
        long pivot = arr[high]; int i = low - 1;
        for (int j = low; j < high; j++) if (arr[j] < pivot) { i++; long t = arr[i]; arr[i] = arr[j]; arr[j] = t; }
        long t = arr[i+1]; arr[i+1] = arr[high]; arr[high] = t; return i + 1;
    }

    static double benchmarkSorting() {
        int n = 1_000_000;
        long[] arr = new long[n];
        for (int i = 0; i < n; i++) arr[i] = n - i - 1;
        long start = System.currentTimeMillis();
        // Iterative quicksort
        int[] stack = new int[n]; int top = -1;
        stack[++top] = 0; stack[++top] = n - 1;
        while (top >= 0) { int h = stack[top--], l = stack[top--]; if (l < h) { int p = partition(arr, l, h); if (p - 1 > l) { stack[++top] = l; stack[++top] = p - 1; } if (p + 1 < h) { stack[++top] = p + 1; stack[++top] = h; } } }
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("Sorting (1M elements): " + elapsed + " ms");
        return elapsed;
    }

    static int sieveOfEratosthenes(int limit) {
        boolean[] sieve = new boolean[limit + 1];
        java.util.Arrays.fill(sieve, true);
        sieve[0] = sieve[1] = false;
        for (int p = 2; p * p <= limit; p++) if (sieve[p]) for (int i = p * p; i <= limit; i += p) sieve[i] = false;
        int count = 0; for (boolean b : sieve) if (b) count++; return count;
    }

    static double benchmarkPrimeSieve() {
        int limit = 10_000_000;
        long start = System.currentTimeMillis();
        int count = sieveOfEratosthenes(limit);
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("Prime Sieve (" + limit + "): " + count + " primes in " + elapsed + " ms");
        return elapsed;
    }

    static double benchmarkStringProcessing() {
        int n = 100_000;
        long start = System.currentTimeMillis();
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < n; i++) sb.append("Item ").append(i).append(" ");
        long elapsed = System.currentTimeMillis() - start;
        System.out.println("String Concatenation (" + n + "): " + elapsed + " ms");
        return elapsed;
    }

    public static void main(String[] args) {
        System.out.println("=== Java CPU Benchmark (Fixed) ===\n");
        double total = 0;
        total += benchmarkFibonacci();
        total += benchmarkMatrixMultiply();
        total += benchmarkSorting();
        total += benchmarkPrimeSieve();
        total += benchmarkStringProcessing();
        System.out.println("\nTotal: " + total + " ms");
    }
}
