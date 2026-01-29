# HTTP in Nostos

## HTTP Client

```nostos
# Simple GET request
response = Http.get("https://api.example.com/users")
println(response.body)

# With headers
response = Http.get("https://api.example.com/users", %{
    "Authorization": "Bearer token123",
    "Accept": "application/json"
})

# POST with JSON body
response = Http.post("https://api.example.com/users",
    json.stringify(%{"name": "Alice", "email": "alice@example.com"}),
    %{"Content-Type": "application/json"}
)

# Other HTTP methods
Http.put(url, body, headers)
Http.patch(url, body, headers)
Http.delete(url, headers)
```

## Response Handling

```nostos
response = Http.get("https://api.example.com/data")

# Check status
if response.status == 200 then {
    data = json.parse(response.body)
    processData(data)
} else {
    println("Error: " ++ show(response.status))
}

# Response fields
response.status      # Int: HTTP status code
response.body        # String: response body
response.headers     # Map[String, String]: response headers
```

## HTTP Server

```nostos
use stdlib.server.*

# Basic server
handler(req) = {
    match req.path {
        "/" -> respondText(req, "Hello, World!")
        "/health" -> respondText(req, "OK")
        _ -> respond404(req)
    }
}

main() = serve(8080, handler)
```

## Request Object

```nostos
handler(req) = {
    # Request fields
    req.path        # String: URL path
    req.method      # String: GET, POST, etc.
    req.body        # String: request body
    req.headers     # Map[String, String]: request headers
    req.query       # Map[String, String]: query parameters
    req.id          # Int: unique request ID for responding
}
```

## Response Helpers

```nostos
use stdlib.server.*

handler(req) = {
    # Text response
    respondText(req, "Hello")

    # JSON response
    respondJson(req, %{"status": "ok", "count": 42})

    # HTML response
    respondHtml(req, "<h1>Hello</h1>")

    # Custom status and headers
    respond(req, 201, %{"X-Custom": "value"}, "Created")

    # Error responses
    respond404(req)
    respond500(req, "Internal error")
}
```

## Route Matching

```nostos
use stdlib.server.*

handler(req) = match req.path {
    "/" -> respondText(req, "Home")
    "/api/users" -> handleUsers(req)
    "/api/posts" -> handlePosts(req)
    path when path.startsWith("/static/") -> serveStatic(req)
    _ -> respond404(req)
}

# With path parameters (manual parsing)
handleUserById(req) = {
    # /users/123 -> extract "123"
    parts = req.path.split("/")
    userId = parts[2].toInt()
    user = findUser(userId)
    respondJson(req, user)
}
```

## Query Parameters

```nostos
# URL: /search?q=nostos&limit=10
handler(req) = {
    query = req.query.get("q").getOrElse("")
    limit = req.query.get("limit").map(s => s.toInt()).getOrElse(20)

    results = search(query, limit)
    respondJson(req, results)
}
```

## JSON API Pattern

```nostos
use stdlib.server.*

type User = { id: Int, name: String, email: String }

# GET /users
getUsers(req) = {
    users = fetchAllUsers()
    respondJson(req, users)
}

# POST /users
createUser(req) = {
    data = json.parse(req.body)
    user = User(
        id: generateId(),
        name: data["name"],
        email: data["email"]
    )
    saveUser(user)
    respond(req, 201, %{}, json.stringify(user))
}

# Router
handler(req) = match (req.method, req.path) {
    ("GET", "/users") -> getUsers(req)
    ("POST", "/users") -> createUser(req)
    _ -> respond404(req)
}
```

## Middleware Pattern

```nostos
# Logging middleware
withLogging(handler) = req => {
    println(req.method ++ " " ++ req.path)
    start = currentTimeMillis()
    result = handler(req)
    elapsed = currentTimeMillis() - start
    println("Completed in " ++ show(elapsed) ++ "ms")
    result
}

# Auth middleware
withAuth(handler) = req => {
    token = req.headers.get("Authorization")
    match token {
        Some(t) when isValidToken(t) -> handler(req)
        _ -> respond(req, 401, %{}, "Unauthorized")
    }
}

# Compose middlewares
main() = {
    handler = withLogging(withAuth(apiHandler))
    serve(8080, handler)
}
```

## Error Handling

```nostos
handler(req) = {
    try {
        data = processRequest(req)
        respondJson(req, data)
    } catch {
        "not found" -> respond404(req)
        "unauthorized" -> respond(req, 401, %{}, "Unauthorized")
        e -> respond500(req, "Error: " ++ e)
    }
}
```

## Concurrent Requests

```nostos
# Fetch multiple URLs in parallel
fetchAll(urls: List[String]) -> List[String] = {
    me = self()

    # Spawn request for each URL
    urls.each((url, i) => spawn(() => {
        response = Http.get(url)
        send(me, (i, response.body))
    }))

    # Collect results in order
    results = urls.map(_ => receive())
    results.sortBy(r => r.0).map(r => r.1)
}
```

## Graceful Shutdown

```nostos
mvar running: Bool = true

handler(req) = match req.path {
    "/shutdown" -> {
        running = false
        respondText(req, "Shutting down...")
    }
    _ -> respondText(req, "Hello")
}

serverLoop(handle) = {
    if running then {
        req = Server.accept(handle)
        spawn(() => handler(req))
        serverLoop(handle)
    } else {
        Server.close(handle)
    }
}

main() = {
    handle = Server.bind(8080)
    println("Server running on :8080")
    serverLoop(handle)
}
```

## CORS Headers

```nostos
withCors(handler) = req => {
    corsHeaders = %{
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE",
        "Access-Control-Allow-Headers": "Content-Type, Authorization"
    }

    if req.method == "OPTIONS" then
        respond(req, 204, corsHeaders, "")
    else {
        response = handler(req)
        # Add CORS headers to response
        response
    }
}
```
