/**
 * Ruva HTTP Server — Java Reference Implementation
 * 
 * A lightweight, non-blocking HTTP server demonstrating patterns
 * that Ruva's stdlib/server module aims to provide natively.
 * 
 * Features:
 *   - TCP socket-based HTTP/1.1 server
 *   - Request routing with path parameters
 *   - Middleware pipeline (CORS, logging, auth)
 *   - JSON request/response handling
 *   - Thread pool for concurrent connections
 *   - Graceful shutdown
 * 
 * Usage:
 *   javac HttpServer.java && java HttpServer
 *   curl http://localhost:8080/
 *   curl http://localhost:8080/api/users/42
 */

import java.io.*;
import java.net.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.*;
import java.time.Instant;

public class HttpServer {

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP Types
    // ═══════════════════════════════════════════════════════════════════════════

    public enum Method { GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD }

    public static class Request {
        public final Method method;
        public final String path;
        public final Map<String, String> headers;
        public final Map<String, String> queryParams;
        public final Map<String, String> pathParams;
        public final String body;

        public Request(Method method, String path, Map<String, String> headers,
                       Map<String, String> queryParams, String body) {
            this.method = method;
            this.path = path;
            this.headers = Collections.unmodifiableMap(headers);
            this.queryParams = Collections.unmodifiableMap(queryParams);
            this.pathParams = new HashMap<>();
            this.body = body;
        }
    }

    public static class Response {
        private int statusCode = 200;
        private final Map<String, String> headers = new LinkedHashMap<>();
        private String body = "";

        public Response status(int code) {
            this.statusCode = code;
            return this;
        }

        public Response header(String key, String value) {
            headers.put(key, value);
            return this;
        }

        public Response json(String json) {
            header("Content-Type", "application/json");
            this.body = json;
            return this;
        }

        public Response text(String text) {
            header("Content-Type", "text/plain; charset=utf-8");
            this.body = text;
            return this;
        }

        public Response html(String html) {
            header("Content-Type", "text/html; charset=utf-8");
            this.body = html;
            return this;
        }

        public String build() {
            StringBuilder sb = new StringBuilder();
            sb.append("HTTP/1.1 ").append(statusCode);
            sb.append(" ").append(statusMessage(statusCode)).append("\r\n");
            headers.put("Content-Length", String.valueOf(body.length()));
            for (Map.Entry<String, String> h : headers.entrySet()) {
                sb.append(h.getKey()).append(": ").append(h.getValue()).append("\r\n");
            }
            sb.append("\r\n").append(body);
            return sb.toString();
        }

        private String statusMessage(int code) {
            return switch (code) {
                case 200 -> "OK";
                case 201 -> "Created";
                case 204 -> "No Content";
                case 301 -> "Moved Permanently";
                case 400 -> "Bad Request";
                case 401 -> "Unauthorized";
                case 403 -> "Forbidden";
                case 404 -> "Not Found";
                case 405 -> "Method Not Allowed";
                case 500 -> "Internal Server Error";
                default -> "Unknown";
            };
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Router
    // ═══════════════════════════════════════════════════════════════════════════

    @FunctionalInterface
    public interface Handler {
        void handle(Request req, Response res);
    }

    public static class Route {
        final Method method;
        final String pattern;
        final Handler handler;
        final String[] paramNames;

        Route(Method method, String pattern, Handler handler) {
            this.method = method;
            this.pattern = pattern;
            this.handler = handler;
            this.paramNames = extractParamNames(pattern);
        }

        boolean matches(Method method, String path) {
            return this.method == method && matchPath(path) != null;
        }

        Map<String, String> matchPath(String path) {
            String[] patternParts = pattern.split("/");
            String[] pathParts = path.split("/");
            if (patternParts.length != pathParts.length) return null;

            Map<String, String> params = new HashMap<>();
            for (int i = 0; i < patternParts.length; i++) {
                if (patternParts[i].startsWith("{") && patternParts[i].endsWith("}")) {
                    params.put(patternParts[i].substring(1, patternParts[i].length() - 1), pathParts[i]);
                } else if (!patternParts[i].equals(pathParts[i])) {
                    return null;
                }
            }
            return params;
        }

        private String[] extractParamNames(String pattern) {
            List<String> names = new ArrayList<>();
            for (String part : pattern.split("/")) {
                if (part.startsWith("{") && part.endsWith("}")) {
                    names.add(part.substring(1, part.length() - 1));
                }
            }
            return names.toArray(new String[0]);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Middleware
    // ═══════════════════════════════════════════════════════════════════════════

    @FunctionalInterface
    public interface Middleware {
        void apply(Request req, Response res, Runnable next);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Server Core
    // ═══════════════════════════════════════════════════════════════════════════

    public static class Server {
        private final List<Route> routes = new ArrayList<>();
        private final List<Middleware> middlewares = new ArrayList<>();
        private final ExecutorService executor;
        private ServerSocket serverSocket;
        private volatile boolean running;

        public Server(int port, int threads) {
            this.executor = Executors.newFixedThreadPool(threads);
            try {
                this.serverSocket = new ServerSocket(port);
            } catch (IOException e) {
                throw new RuntimeException("Failed to bind port " + port, e);
            }
        }

        public Server get(String path, Handler handler) {
            routes.add(new Route(Method.GET, path, handler));
            return this;
        }

        public Server post(String path, Handler handler) {
            routes.add(new Route(Method.POST, path, handler));
            return this;
        }

        public Server put(String path, Handler handler) {
            routes.add(new Route(Method.PUT, path, handler));
            return this;
        }

        public Server delete(String path, Handler handler) {
            routes.add(new Route(Method.DELETE, path, handler));
            return this;
        }

        public Server use(Middleware middleware) {
            middlewares.add(middleware);
            return this;
        }

        public void start() {
            running = true;
            System.out.println("[Server] Listening on " + serverSocket.getLocalSocketAddress());

            while (running) {
                try {
                    Socket socket = serverSocket.accept();
                    executor.submit(() -> handleConnection(socket));
                } catch (IOException e) {
                    if (running) System.err.println("[Server] Accept error: " + e.getMessage());
                }
            }
        }

        public void stop() {
            running = false;
            executor.shutdown();
            try { serverSocket.close(); } catch (IOException ignored) {}
        }

        private void handleConnection(Socket socket) {
            try (socket; BufferedReader in = new BufferedReader(new InputStreamReader(socket.getInputStream()));
                 OutputStream out = socket.getOutputStream()) {

                // Parse request line
                String requestLine = in.readLine();
                if (requestLine == null || requestLine.isEmpty()) return;

                String[] parts = requestLine.split(" ");
                Method method = Method.valueOf(parts[0]);
                String fullPath = parts[1];

                // Parse query params
                Map<String, String> queryParams = new HashMap<>();
                String path = fullPath;
                if (fullPath.contains("?")) {
                    path = fullPath.substring(0, fullPath.indexOf("?"));
                    String queryString = fullPath.substring(fullPath.indexOf("?") + 1);
                    for (String param : queryString.split("&")) {
                        String[] kv = param.split("=", 2);
                        queryParams.put(URLDecoder.decode(kv[0], "UTF-8"),
                                        kv.length > 1 ? URLDecoder.decode(kv[1], "UTF-8") : "");
                    }
                }

                // Parse headers
                Map<String, String> headers = new HashMap<>();
                String headerLine;
                while ((headerLine = in.readLine()) != null && !headerLine.isEmpty()) {
                    int colon = headerLine.indexOf(':');
                    if (colon > 0) {
                        headers.put(headerLine.substring(0, colon).trim().toLowerCase(),
                                    headerLine.substring(colon + 1).trim());
                    }
                }

                // Read body
                String body = "";
                if (headers.containsKey("content-length")) {
                    int len = Integer.parseInt(headers.get("content-length"));
                    char[] bodyChars = new char[len];
                    int read = 0;
                    while (read < len) {
                        read += in.read(bodyChars, read, len - read);
                    }
                    body = new String(bodyChars);
                }

                Request req = new Request(method, path, headers, queryParams, body);
                Response res = new Response();

                // Run middleware chain
                runMiddleware(req, res, 0, () -> dispatch(req, res));

                // Write response
                out.write(res.build().getBytes());
                out.flush();

            } catch (Exception e) {
                System.err.println("[Server] Error: " + e.getMessage());
            }
        }

        private void runMiddleware(Request req, Response res, int index, Runnable finalHandler) {
            if (index >= middlewares.size()) {
                finalHandler.run();
                return;
            }
            middlewares.get(index).apply(req, res, () ->
                runMiddleware(req, res, index + 1, finalHandler));
        }

        private void dispatch(Request req, Response res) {
            for (Route route : routes) {
                if (route.matches(req.method, req.path)) {
                    Map<String, String> params = route.matchPath(req.path);
                    if (params != null) {
                        req.pathParams.putAll(params);
                        route.handler.handle(req, res);
                        return;
                    }
                }
            }
            res.status(404).text("Not Found");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Built-in Middleware
    // ═══════════════════════════════════════════════════════════════════════════

    /** CORS middleware — allows cross-origin requests */
    public static Middleware corsMiddleware(String allowOrigin) {
        return (req, res, next) -> {
            res.header("Access-Control-Allow-Origin", allowOrigin);
            res.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
            res.header("Access-Control-Allow-Headers", "Content-Type, Authorization");
            if (req.method == Method.OPTIONS) {
                res.status(204);
                return;
            }
            next.run();
        };
    }

    /** Request logging middleware */
    public static Middleware loggingMiddleware() {
        return (req, res, next) -> {
            Instant start = Instant.now();
            next.run();
            long ms = Instant.now().toEpochMilli() - start.toEpochMilli();
            System.out.printf("[%s] %s %s → %d (%dms)%n",
                Instant.now(), req.method, req.path, res.statusCode, ms);
        };
    }

    /** Simple API key authentication middleware */
    public static Middleware authMiddleware(String validKey) {
        return (req, res, next) -> {
            String auth = req.headers.getOrDefault("authorization", "");
            if (!auth.equals("Bearer " + validKey)) {
                res.status(401).json("{\"error\": \"Unauthorized\"}");
                return;
            }
            next.run();
        };
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Demo Application
    // ═══════════════════════════════════════════════════════════════════════════

    public static void main(String[] args) {
        System.out.println("=== Ruva HTTP Server (Java Reference) ===\n");

        int port = args.length > 0 ? Integer.parseInt(args[0]) : 8080;

        Server server = new Server(port, 4);

        // Middleware
        server.use(corsMiddleware("*"));
        server.use(loggingMiddleware());

        // Routes
        server.get("/", (req, res) ->
            res.html("<h1>Ruva HTTP Server</h1><p>Java reference implementation</p>"));

        server.get("/api/health", (req, res) ->
            res.json("{\"status\": \"ok\", \"lang\": \"java\", \"server\": \"ruva-ref\"}"));

        server.get("/api/users/{id}", (req, res) ->
            res.json("{\"id\": " + req.pathParams.get("id") + ", \"name\": \"User " +
                      req.pathParams.get("id") + "\"}"));

        server.post("/api/echo", (req, res) ->
            res.json("{\"echo\": \"" + req.body.replace("\"", "\\\"") + "\"}"));

        server.get("/api/time", (req, res) ->
            res.json("{\"time\": \"" + Instant.now().toString() + "\"}"));

        // Start
        server.start();
    }
}
