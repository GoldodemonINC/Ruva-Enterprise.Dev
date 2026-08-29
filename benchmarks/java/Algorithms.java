/**
 * Ruva Algorithms Library — Java Reference Implementations
 * 
 * Common algorithms for benchmarking and reference:
 *   - Sorting: QuickSort, MergeSort, HeapSort, RadixSort
 *   - Searching: Binary Search, KMP String Matching
 *   - Graph: BFS, DFS, Dijkstra, Topological Sort
 *   - Math: GCD, Fast Exponentiation, Modular Arithmetic
 */

import java.util.*;

public class Algorithms {

    // ═══════════════════════════════════════════════════════════════════════════
    // Sorting Algorithms
    // ═══════════════════════════════════════════════════════════════════════════

    /** QuickSort — O(n log n) average, O(n²) worst case */
    public static void quickSort(int[] arr, int low, int high) {
        if (low < high) {
            int pivot = partition(arr, low, high);
            quickSort(arr, low, pivot - 1);
            quickSort(arr, pivot + 1, high);
        }
    }

    private static int partition(int[] arr, int low, int high) {
        int pivot = arr[high];
        int i = low - 1;
        for (int j = low; j < high; j++) {
            if (arr[j] < pivot) {
                i++;
                swap(arr, i, j);
            }
        }
        swap(arr, i + 1, high);
        return i + 1;
    }

    /** MergeSort — O(n log n) guaranteed */
    public static void mergeSort(int[] arr, int left, int right) {
        if (left < right) {
            int mid = left + (right - left) / 2;
            mergeSort(arr, left, mid);
            mergeSort(arr, mid + 1, right);
            merge(arr, left, mid, right);
        }
    }

    private static void merge(int[] arr, int left, int mid, int right) {
        int n1 = mid - left + 1;
        int n2 = right - mid;
        int[] L = new int[n1];
        int[] R = new int[n2];
        System.arraycopy(arr, left, L, 0, n1);
        System.arraycopy(arr, mid + 1, R, 0, n2);

        int i = 0, j = 0, k = left;
        while (i < n1 && j < n2) {
            arr[k++] = (L[i] <= R[j]) ? L[i++] : R[j++];
        }
        while (i < n1) arr[k++] = L[i++];
        while (j < n2) arr[k++] = R[j++];
    }

    /** HeapSort — O(n log n) in-place */
    public static void heapSort(int[] arr) {
        int n = arr.length;
        for (int i = n / 2 - 1; i >= 0; i--) heapify(arr, n, i);
        for (int i = n - 1; i > 0; i--) {
            swap(arr, 0, i);
            heapify(arr, i, 0);
        }
    }

    private static void heapify(int[] arr, int n, int i) {
        int largest = i, left = 2 * i + 1, right = 2 * i + 2;
        if (left < n && arr[left] > arr[largest]) largest = left;
        if (right < n && arr[right] > arr[largest]) largest = right;
        if (largest != i) {
            swap(arr, i, largest);
            heapify(arr, n, largest);
        }
    }

    /** RadixSort — O(d × (n + b)) where d = digits, b = base */
    public static void radixSort(int[] arr) {
        int max = Arrays.stream(arr).max().orElse(0);
        for (int exp = 1; max / exp > 0; exp *= 10) {
            countingSortByDigit(arr, exp);
        }
    }

    private static void countingSortByDigit(int[] arr, int exp) {
        int n = arr.length;
        int[] output = new int[n];
        int[] count = new int[10];

        for (int val : arr) count[(val / exp) % 10]++;
        for (int i = 1; i < 10; i++) count[i] += count[i - 1];
        for (int i = n - 1; i >= 0; i--) {
            output[--count[(arr[i] / exp) % 10]] = arr[i];
        }
        System.arraycopy(output, 0, arr, 0, n);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Searching Algorithms
    // ═══════════════════════════════════════════════════════════════════════════

    /** Binary Search — O(log n) on sorted array */
    public static int binarySearch(int[] arr, int target) {
        int lo = 0, hi = arr.length - 1;
        while (lo <= hi) {
            int mid = lo + (hi - lo) / 2;
            if (arr[mid] == target) return mid;
            else if (arr[mid] < target) lo = mid + 1;
            else hi = mid - 1;
        }
        return -1;
    }

    /** KMP String Search — O(n + m) */
    public static int kmpSearch(String text, String pattern) {
        int[] lps = computeLPS(pattern);
        int i = 0, j = 0;
        while (i < text.length()) {
            if (text.charAt(i) == pattern.charAt(j)) {
                i++;
                j++;
                if (j == pattern.length()) return i - j;
            } else if (j > 0) {
                j = lps[j - 1];
            } else {
                i++;
            }
        }
        return -1;
    }

    private static int[] computeLPS(String pattern) {
        int[] lps = new int[pattern.length()];
        int len = 0, i = 1;
        while (i < pattern.length()) {
            if (pattern.charAt(i) == pattern.charAt(len)) {
                lps[i++] = ++len;
            } else if (len > 0) {
                len = lps[len - 1];
            } else {
                lps[i++] = 0;
            }
        }
        return lps;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Graph Algorithms
    // ═══════════════════════════════════════════════════════════════════════════

    /** BFS — Breadth-First Search */
    public static List<Integer> bfs(List<List<Integer>> adj, int start) {
        List<Integer> result = new ArrayList<>();
        boolean[] visited = new boolean[adj.size()];
        Queue<Integer> queue = new LinkedList<>();
        visited[start] = true;
        queue.add(start);

        while (!queue.isEmpty()) {
            int node = queue.poll();
            result.add(node);
            for (int neighbor : adj.get(node)) {
                if (!visited[neighbor]) {
                    visited[neighbor] = true;
                    queue.add(neighbor);
                }
            }
        }
        return result;
    }

    /** DFS — Depth-First Search (recursive) */
    public static List<Integer> dfs(List<List<Integer>> adj, int start) {
        List<Integer> result = new ArrayList<>();
        boolean[] visited = new boolean[adj.size()];
        dfsHelper(adj, start, visited, result);
        return result;
    }

    private static void dfsHelper(List<List<Integer>> adj, int node,
                                   boolean[] visited, List<Integer> result) {
        visited[node] = true;
        result.add(node);
        for (int neighbor : adj.get(node)) {
            if (!visited[neighbor]) {
                dfsHelper(adj, neighbor, visited, result);
            }
        }
    }

    /** Dijkstra's Shortest Path — O((V + E) log V) */
    public static int[] dijkstra(List<List<int[]>> adj, int src) {
        int n = adj.size();
        int[] dist = new int[n];
        Arrays.fill(dist, Integer.MAX_VALUE);
        dist[src] = 0;

        PriorityQueue<int[]> pq = new PriorityQueue<>(Comparator.comparingInt(a -> a[1]));
        pq.add(new int[]{src, 0});

        while (!pq.isEmpty()) {
            int[] curr = pq.poll();
            int u = curr[0], d = curr[1];
            if (d > dist[u]) continue;
            for (int[] edge : adj.get(u)) {
                int v = edge[0], w = edge[1];
                if (dist[u] + w < dist[v]) {
                    dist[v] = dist[u] + w;
                    pq.add(new int[]{v, dist[v]});
                }
            }
        }
        return dist;
    }

    /** Topological Sort — Kahn's algorithm */
    public static List<Integer> topologicalSort(List<List<Integer>> adj) {
        int n = adj.size();
        int[] inDegree = new int[n];
        for (int u = 0; u < n; u++) {
            for (int v : adj.get(u)) inDegree[v]++;
        }

        Queue<Integer> queue = new LinkedList<>();
        for (int i = 0; i < n; i++) {
            if (inDegree[i] == 0) queue.add(i);
        }

        List<Integer> result = new ArrayList<>();
        while (!queue.isEmpty()) {
            int u = queue.poll();
            result.add(u);
            for (int v : adj.get(u)) {
                if (--inDegree[v] == 0) queue.add(v);
            }
        }

        if (result.size() != n) {
            throw new IllegalStateException("Graph has a cycle");
        }
        return result;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Math Utilities
    // ═══════════════════════════════════════════════════════════════════════════

    /** Greatest Common Divisor */
    public static long gcd(long a, long b) {
        while (b != 0) {
            long temp = b;
            b = a % b;
            a = temp;
        }
        return a;
    }

    /** Fast modular exponentiation — O(log n) */
    public static long modPow(long base, long exp, long mod) {
        long result = 1;
        base %= mod;
        while (exp > 0) {
            if ((exp & 1) == 1) {
                result = (result * base) % mod;
            }
            exp >>= 1;
            base = (base * base) % mod;
        }
        return result;
    }

    /** Extended Euclidean Algorithm — returns {gcd, x, y} such that ax + by = gcd */
    public static long[] extendedGcd(long a, long b) {
        if (b == 0) return new long[]{a, 1, 0};
        long[] prev = extendedGcd(b, a % b);
        long gcd = prev[0];
        long x = prev[2];
        long y = prev[1] - (a / b) * prev[2];
        return new long[]{gcd, x, y};
    }

    /** Sieve of Eratosthenes — returns list of primes up to limit */
    public static List<Integer> sieveOfEratosthenes(int limit) {
        boolean[] sieve = new boolean[limit + 1];
        Arrays.fill(sieve, true);
        sieve[0] = sieve[1] = false;

        for (int p = 2; p * p <= limit; p++) {
            if (sieve[p]) {
                for (int i = p * p; i <= limit; i += p) {
                    sieve[i] = false;
                }
            }
        }

        List<Integer> primes = new ArrayList<>();
        for (int i = 2; i <= limit; i++) {
            if (sieve[i]) primes.add(i);
        }
        return primes;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Helpers
    // ═══════════════════════════════════════════════════════════════════════════

    private static void swap(int[] arr, int i, int j) {
        int tmp = arr[i];
        arr[i] = arr[j];
        arr[j] = tmp;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Demo / Test
    // ═══════════════════════════════════════════════════════════════════════════

    public static void main(String[] args) {
        System.out.println("=== Ruva Algorithms Library (Java) ===\n");

        // Sorting tests
        int[] data = {38, 27, 43, 3, 9, 82, 10};
        int[] copy;

        copy = data.clone();
        quickSort(copy, 0, copy.length - 1);
        System.out.println("QuickSort:    " + Arrays.toString(copy));

        copy = data.clone();
        mergeSort(copy, 0, copy.length - 1);
        System.out.println("MergeSort:    " + Arrays.toString(copy));

        copy = data.clone();
        heapSort(copy);
        System.out.println("HeapSort:     " + Arrays.toString(copy));

        copy = data.clone();
        radixSort(copy);
        System.out.println("RadixSort:    " + Arrays.toString(copy));

        // Searching
        int[] sorted = {1, 3, 5, 7, 9, 11, 13};
        System.out.println("\nBinarySearch(7): index=" + binarySearch(sorted, 7));
        System.out.println("KMP('ABABD', 'ABD'): index=" + kmpSearch("ABABDABACDABABCABAB", "ABD"));

        // Graph
        List<List<Integer>> graph = new ArrayList<>();
        for (int i = 0; i < 5; i++) graph.add(new ArrayList<>());
        graph.get(0).addAll(Arrays.asList(1, 2));
        graph.get(1).addAll(Arrays.asList(3, 4));
        graph.get(2).add(4);

        System.out.println("\nBFS from 0:  " + bfs(graph, 0));
        System.out.println("DFS from 0:  " + dfs(graph, 0));

        // Math
        System.out.println("\ngcd(48, 18):  " + gcd(48, 18));
        System.out.println("modPow(2,10,1000): " + modPow(2, 10, 1000));
        System.out.println("Primes <= 50: " + sieveOfEratosthenes(50).size() + " primes");

        System.out.println("\nAll algorithms functional.");
    }
}
