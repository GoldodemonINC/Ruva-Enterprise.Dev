// Security Benchmark - Java
// Tests: bounds checking, null safety, overflow detection, memory safety

import java.util.Arrays;

public class SecurityBenchmark {

    // ─── Bounds Checking ────────────────────────────────────────────────────

    static double benchmarkBoundsChecking(int n) {
        long[] data = new long[n];
        for (int i = 0; i < n; i++) data[i] = i;

        long start = System.nanoTime();
        long sum = 0;
        for (int i = 0; i < n; i++) {
            // Bounds-checked access — throws ArrayIndexOutOfBoundsException
            sum += data[i];
        }
        double elapsed = (System.nanoTime() - start) / 1000.0;

        System.out.printf("Bounds-Checked Array Access (%d elements): %.2f µs%n", n, elapsed);
        return elapsed;
    }

    // ─── Null Safety (manual — no Option<T>) ────────────────────────────────

    static int findValue(int[] data, int target) {
        for (int i = 0; i < data.length; i++) {
            if (data[i] == target) return i;
        }
        return -1;  // Manual sentinel for "not found" — error-prone
    }

    static double benchmarkNullSafety(int n) {
        int[] data = new int[n];
        for (int i = 0; i < n; i++) data[i] = i;

        long start = System.nanoTime();
        long found = 0;
        for (int target = 0; target < n; target++) {
            if (findValue(data, target) >= 0) found++;
            if (findValue(data, n + target) < 0) found++;
        }
        double elapsed = (System.nanoTime() - start) / 1000.0;

        System.out.printf("Null Safety (sentinel) Searches (%d lookups): %.2f µs%n", n * 2, elapsed);
        return elapsed;
    }

    // ─── Integer Overflow Detection (manual) ────────────────────────────────

    static Long safeAdd(long a, long b) {
        long result = a + b;
        // Manual overflow check — easy to forget!
        if (((a ^ result) & (b ^ result)) < 0) {
            return null;  // Overflow detected (manual)
        }
        return result;
    }

    static double benchmarkOverflowDetection(int n) {
        long start = System.nanoTime();
        long count = 0;
        for (int i = 0; i < n; i++) {
            if (safeAdd(i, i) != null) count++;
            if (safeAdd(Long.MAX_VALUE - i, i + 1) == null) count++;
        }
        double elapsed = (System.nanoTime() - start) / 1000.0;

        System.out.printf("Overflow Detection (%d checks): %.2f µs%n", n * 2, elapsed);
        return elapsed;
    }

    // ─── String Safety ──────────────────────────────────────────────────────

    static double benchmarkStringSafety(int iterations) {
        long start = System.nanoTime();
        int totalLen = 0;
        for (int i = 0; i < iterations; i++) {
            // Java strings are UTF-16 — safe from buffer overflows
            String s = String.format("Item %d: 安全なUTF-8文字列 🔒🛡️", i);
            totalLen += s.length();
            // charAt is bounds-checked
            if (s.length() > 0) {
                char last = s.charAt(s.length() - 1);
            }
        }
        double elapsed = (System.nanoTime() - start) / 1000.0;

        System.out.printf("String Safety (%d UTF ops): %.2f µs%n", iterations, elapsed);
        return elapsed;
    }

    // ─── Memory Safety (GC-managed — no use-after-free) ─────────────────────

    static double benchmarkMemorySafety(int iterations) {
        long start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            // GC prevents use-after-free — but adds pause overhead
            long[] data = new long[100];
            Arrays.fill(data, (long) i);
            long sum = 0;
            for (long v : data) sum += v;
        }
        double elapsed = (System.nanoTime() - start) / 1000.0;

        System.out.printf("Memory Safety (GC alloc/dealloc, %d iter): %.2f µs%n", iterations, elapsed);
        return elapsed;
    }

    // ─── Main ───────────────────────────────────────────────────────────────

    public static void main(String[] args) {
        System.out.println("=== Java Security Benchmark ===");
        System.out.println();
        System.out.println("Security analysis:");
        System.out.println("  ✓ Bounds checking on array access (runtime)");
        System.out.println("  ✗ Null safety — uses sentinel values, NullPointerExceptions possible");
        System.out.println("  ✗ Integer overflow — silent wraparound in Java (unchecked)");
        System.out.println("  ✓ UTF-16 safe string operations");
        System.out.println("  ~ Memory safety via GC (no use-after-free, but GC pauses)");
        System.out.println();

        double total = 0;

        total += benchmarkBoundsChecking(1_000_000);
        total += benchmarkNullSafety(10_000);
        total += benchmarkOverflowDetection(100_000);
        total += benchmarkStringSafety(100_000);
        total += benchmarkMemorySafety(100_000);

        System.out.println();
        System.out.printf("Total: %.2f µs (%.2f ms)%n", total, total / 1000.0);
        System.out.println();
        System.out.println("Security guarantees: PARTIAL (runtime checks only, no compile-time safety)");
    }
}
