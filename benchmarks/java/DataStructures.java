/**
 * Ruva Data Structures Library — Java Reference Implementations
 * 
 * This file contains core data structure implementations that mirror
 * what Ruva's stdlib provides. Used for:
 *   1. Cross-language benchmarking
 *   2. Reference implementation for Ruva transpiler output
 *   3. Algorithm testing
 * 
 * Structures: ArrayList, LinkedList, HashMap, Stack, Queue, BinaryTree, Heap
 */

import java.util.*;
import java.util.function.*;

public class DataStructures {

    // ═══════════════════════════════════════════════════════════════════════════
    // ArrayList — Dynamic array with automatic resizing
    // ═══════════════════════════════════════════════════════════════════════════

    public static class ArrayList<T> {
        private Object[] data;
        private int size;
        private static final int DEFAULT_CAPACITY = 16;

        public ArrayList() {
            this.data = new Object[DEFAULT_CAPACITY];
            this.size = 0;
        }

        public ArrayList(int initialCapacity) {
            this.data = new Object[Math.max(initialCapacity, 1)];
            this.size = 0;
        }

        public void add(T item) {
            ensureCapacity(size + 1);
            data[size++] = item;
        }

        public void add(int index, T item) {
            rangeCheckForAdd(index);
            ensureCapacity(size + 1);
            System.arraycopy(data, index, data, index + 1, size - index);
            data[index] = item;
            size++;
        }

        @SuppressWarnings("unchecked")
        public T get(int index) {
            rangeCheck(index);
            return (T) data[index];
        }

        @SuppressWarnings("unchecked")
        public T set(int index, T item) {
            rangeCheck(index);
            T old = (T) data[index];
            data[index] = item;
            return old;
        }

        @SuppressWarnings("unchecked")
        public T remove(int index) {
            rangeCheck(index);
            T old = (T) data[index];
            int numMoved = size - index - 1;
            if (numMoved > 0) {
                System.arraycopy(data, index + 1, data, index, numMoved);
            }
            data[--size] = null;
            return old;
        }

        public boolean remove(T item) {
            for (int i = 0; i < size; i++) {
                if (Objects.equals(data[i], item)) {
                    remove(i);
                    return true;
                }
            }
            return false;
        }

        public int size() { return size; }
        public boolean isEmpty() { return size == 0; }

        public void clear() {
            for (int i = 0; i < size; i++) data[i] = null;
            size = 0;
        }

        private void ensureCapacity(int minCapacity) {
            if (minCapacity > data.length) {
                int newCapacity = data.length + (data.length >> 1);
                data = Arrays.copyOf(data, Math.max(newCapacity, minCapacity));
            }
        }

        private void rangeCheck(int index) {
            if (index < 0 || index >= size)
                throw new IndexOutOfBoundsException("Index: " + index + ", Size: " + size);
        }

        private void rangeCheckForAdd(int index) {
            if (index < 0 || index > size)
                throw new IndexOutOfBoundsException("Index: " + index + ", Size: " + size);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LinkedList — Doubly-linked list
    // ═══════════════════════════════════════════════════════════════════════════

    public static class LinkedList<T> {
        private static class Node<T> {
            T data;
            Node<T> next;
            Node<T> prev;

            Node(T data) {
                this.data = data;
            }
        }

        private Node<T> head;
        private Node<T> tail;
        private int size;

        public LinkedList() {
            head = null;
            tail = null;
            size = 0;
        }

        public void addFirst(T item) {
            Node<T> node = new Node<>(item);
            if (head == null) {
                head = tail = node;
            } else {
                node.next = head;
                head.prev = node;
                head = node;
            }
            size++;
        }

        public void addLast(T item) {
            Node<T> node = new Node<>(item);
            if (tail == null) {
                head = tail = node;
            } else {
                tail.next = node;
                node.prev = tail;
                tail = node;
            }
            size++;
        }

        public T removeFirst() {
            if (head == null) throw new NoSuchElementException();
            T data = head.data;
            head = head.next;
            if (head == null) tail = null;
            else head.prev = null;
            size--;
            return data;
        }

        public T removeLast() {
            if (tail == null) throw new NoSuchElementException();
            T data = tail.data;
            tail = tail.prev;
            if (tail == null) head = null;
            else tail.next = null;
            size--;
            return data;
        }

        public T getFirst() {
            if (head == null) throw new NoSuchElementException();
            return head.data;
        }

        public T getLast() {
            if (tail == null) throw new NoSuchElementException();
            return tail.data;
        }

        public int size() { return size; }
        public boolean isEmpty() { return size == 0; }

        public boolean contains(T item) {
            Node<T> current = head;
            while (current != null) {
                if (Objects.equals(current.data, item)) return true;
                current = current.next;
            }
            return false;
        }

        public void forEach(Consumer<T> action) {
            Node<T> current = head;
            while (current != null) {
                action.accept(current.data);
                current = current.next;
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HashMap — Hash table with separate chaining
    // ═══════════════════════════════════════════════════════════════════════════

    public static class HashMap<K, V> {
        private static class Entry<K, V> {
            K key;
            V value;
            Entry<K, V> next;
            int hash;

            Entry(K key, V value, int hash) {
                this.key = key;
                this.value = value;
                this.hash = hash;
            }
        }

        private Entry<K, V>[] table;
        private int size;
        private static final int DEFAULT_CAPACITY = 16;
        private static final float DEFAULT_LOAD_FACTOR = 0.75f;

        @SuppressWarnings("unchecked")
        public HashMap() {
            table = new Entry[DEFAULT_CAPACITY];
            size = 0;
        }

        public V put(K key, V value) {
            int hash = key.hashCode();
            int index = indexFor(hash, table.length);

            Entry<K, V> current = table[index];
            while (current != null) {
                if (current.hash == hash && Objects.equals(current.key, key)) {
                    V old = current.value;
                    current.value = value;
                    return old;
                }
                current = current.next;
            }

            Entry<K, V> newEntry = new Entry<>(key, value, hash);
            newEntry.next = table[index];
            table[index] = newEntry;
            size++;

            if (size > table.length * DEFAULT_LOAD_FACTOR) {
                resize();
            }
            return null;
        }

        public V get(K key) {
            int hash = key.hashCode();
            int index = indexFor(hash, table.length);

            Entry<K, V> current = table[index];
            while (current != null) {
                if (current.hash == hash && Objects.equals(current.key, key)) {
                    return current.value;
                }
                current = current.next;
            }
            return null;
        }

        public V remove(K key) {
            int hash = key.hashCode();
            int index = indexFor(hash, table.length);

            Entry<K, V> prev = null;
            Entry<K, V> current = table[index];
            while (current != null) {
                if (current.hash == hash && Objects.equals(current.key, key)) {
                    if (prev == null) table[index] = current.next;
                    else prev.next = current.next;
                    size--;
                    return current.value;
                }
                prev = current;
                current = current.next;
            }
            return null;
        }

        public boolean containsKey(K key) {
            return get(key) != null;
        }

        public int size() { return size; }
        public boolean isEmpty() { return size == 0; }

        @SuppressWarnings("unchecked")
        private void resize() {
            Entry<K, V>[] oldTable = table;
            table = new Entry[oldTable.length * 2];
            size = 0;
            for (Entry<K, V> entry : oldTable) {
                Entry<K, V> current = entry;
                while (current != null) {
                    put(current.key, current.value);
                    current = current.next;
                }
            }
        }

        private int indexFor(int hash, int length) {
            return Math.abs(hash) % length;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Stack — LIFO data structure
    // ═══════════════════════════════════════════════════════════════════════════

    public static class Stack<T> {
        private final LinkedList<T> list = new LinkedList<>();

        public void push(T item) {
            list.addFirst(item);
        }

        public T pop() {
            return list.removeFirst();
        }

        public T peek() {
            return list.getFirst();
        }

        public int size() { return list.size(); }
        public boolean isEmpty() { return list.isEmpty(); }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Queue — FIFO data structure
    // ═══════════════════════════════════════════════════════════════════════════

    public static class Queue<T> {
        private final LinkedList<T> list = new LinkedList<>();

        public void enqueue(T item) {
            list.addLast(item);
        }

        public T dequeue() {
            return list.removeFirst();
        }

        public T peek() {
            return list.getFirst();
        }

        public int size() { return list.size(); }
        public boolean isEmpty() { return list.isEmpty(); }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BinaryTree — Binary search tree
    // ═══════════════════════════════════════════════════════════════════════════

    public static class BinaryTree<T extends Comparable<T>> {
        private static class TreeNode<T> {
            T data;
            TreeNode<T> left, right;

            TreeNode(T data) {
                this.data = data;
            }
        }

        private TreeNode<T> root;
        private int size;

        public BinaryTree() {
            root = null;
            size = 0;
        }

        public void insert(T item) {
            root = insertRec(root, item);
            size++;
        }

        private TreeNode<T> insertRec(TreeNode<T> node, T item) {
            if (node == null) return new TreeNode<>(item);
            if (item.compareTo(node.data) < 0) {
                node.left = insertRec(node.left, item);
            } else if (item.compareTo(node.data) > 0) {
                node.right = insertRec(node.right, item);
            }
            return node;
        }

        public boolean contains(T item) {
            return searchRec(root, item);
        }

        private boolean searchRec(TreeNode<T> node, T item) {
            if (node == null) return false;
            if (item.compareTo(node.data) == 0) return true;
            if (item.compareTo(node.data) < 0) return searchRec(node.left, item);
            return searchRec(node.right, item);
        }

        public List<T> inorder() {
            List<T> result = new ArrayList<>();
            inorderRec(root, result);
            return result;
        }

        private void inorderRec(TreeNode<T> node, List<T> result) {
            if (node == null) return;
            inorderRec(node.left, result);
            result.add(node.data);
            inorderRec(node.right, result);
        }

        public int size() { return size; }

        public int height() {
            return heightRec(root);
        }

        private int heightRec(TreeNode<T> node) {
            if (node == null) return 0;
            return 1 + Math.max(heightRec(node.left), heightRec(node.right));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MinHeap — Priority queue implementation
    // ═══════════════════════════════════════════════════════════════════════════

    public static class MinHeap<T extends Comparable<T>> {
        private ArrayList<T> heap;

        public MinHeap() {
            heap = new ArrayList<>();
        }

        public void add(T item) {
            heap.add(item);
            siftUp(heap.size() - 1);
        }

        public T poll() {
            if (heap.isEmpty()) throw new NoSuchElementException();
            T min = heap.get(0);
            T last = heap.remove(heap.size() - 1);
            if (!heap.isEmpty()) {
                heap.set(0, last);
                siftDown(0);
            }
            return min;
        }

        public T peek() {
            if (heap.isEmpty()) throw new NoSuchElementException();
            return heap.get(0);
        }

        public int size() { return heap.size(); }
        public boolean isEmpty() { return heap.isEmpty(); }

        private void siftUp(int index) {
            while (index > 0) {
                int parent = (index - 1) / 2;
                if (heap.get(index).compareTo(heap.get(parent)) < 0) {
                    swap(index, parent);
                    index = parent;
                } else break;
            }
        }

        private void siftDown(int index) {
            int size = heap.size();
            while (true) {
                int left = 2 * index + 1;
                int right = 2 * index + 2;
                int smallest = index;

                if (left < size && heap.get(left).compareTo(heap.get(smallest)) < 0) {
                    smallest = left;
                }
                if (right < size && heap.get(right).compareTo(heap.get(smallest)) < 0) {
                    smallest = right;
                }
                if (smallest != index) {
                    swap(index, smallest);
                    index = smallest;
                } else break;
            }
        }

        private void swap(int i, int j) {
            T temp = heap.get(i);
            heap.set(i, heap.get(j));
            heap.set(j, temp);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Union-Find — Disjoint set data structure
    // ═══════════════════════════════════════════════════════════════════════════

    public static class UnionFind {
        private int[] parent;
        private int[] rank;
        private int components;

        public UnionFind(int n) {
            parent = new int[n];
            rank = new int[n];
            components = n;
            for (int i = 0; i < n; i++) {
                parent[i] = i;
                rank[i] = 0;
            }
        }

        public int find(int x) {
            if (parent[x] != x) {
                parent[x] = find(parent[x]); // Path compression
            }
            return parent[x];
        }

        public boolean union(int x, int y) {
            int rootX = find(x);
            int rootY = find(y);
            if (rootX == rootY) return false;

            // Union by rank
            if (rank[rootX] < rank[rootY]) {
                parent[rootX] = rootY;
            } else if (rank[rootX] > rank[rootY]) {
                parent[rootY] = rootX;
            } else {
                parent[rootY] = rootX;
                rank[rootX]++;
            }
            components--;
            return true;
        }

        public boolean connected(int x, int y) {
            return find(x) == find(y);
        }

        public int components() { return components; }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Demo / Test
    // ═══════════════════════════════════════════════════════════════════════════

    public static void main(String[] args) {
        System.out.println("=== Ruva Data Structures Library (Java) ===\n");

        // ArrayList test
        ArrayList<String> list = new ArrayList<>();
        list.add("alpha");
        list.add("beta");
        list.add("gamma");
        System.out.println("ArrayList: " + list.get(0) + ", " + list.get(1) + ", " + list.get(2));
        System.out.println("  size: " + list.size());

        // Stack test
        Stack<Integer> stack = new Stack<>();
        stack.push(10);
        stack.push(20);
        stack.push(30);
        System.out.println("\nStack pop: " + stack.pop() + ", " + stack.pop());

        // HashMap test
        HashMap<String, Integer> map = new HashMap<>();
        map.put("one", 1);
        map.put("two", 2);
        map.put("three", 3);
        System.out.println("\nHashMap: one=" + map.get("one") + ", two=" + map.get("two"));

        // BinaryTree test
        BinaryTree<Integer> tree = new BinaryTree<>();
        for (int i : new int[]{5, 3, 7, 1, 4, 6, 8}) {
            tree.insert(i);
        }
        System.out.println("\nBST inorder: " + tree.inorder());
        System.out.println("  height: " + tree.height());

        // MinHeap test
        MinHeap<Integer> heap = new MinHeap<>();
        for (int i : new int[]{5, 3, 7, 1, 4, 6, 8}) {
            heap.add(i);
        }
        System.out.print("\nMinHeap poll order: ");
        while (!heap.isEmpty()) {
            System.out.print(heap.poll() + " ");
        }
        System.out.println();

        // Union-Find test
        UnionFind uf = new UnionFind(6);
        uf.union(0, 1);
        uf.union(1, 2);
        uf.union(3, 4);
        System.out.println("\nUnionFind: 0~2 connected? " + uf.connected(0, 2));
        System.out.println("  0~3 connected? " + uf.connected(0, 3));
        System.out.println("  components: " + uf.components());

        System.out.println("\nAll data structures functional.");
    }
}
