# Nostos End-to-End Examples

Complete working examples for common tasks.

## Read JSON File and Extract Data

```nostos
import file
import json

type User = { id: Int, name: String, email: String }
type UsersFile = { users: List[User] }

main() = {
    # Read and parse JSON file
    content = match file.read("users.json") {
        Ok(s) -> s,
        Err(e) -> {
            println("Failed to read file: " ++ e)
            return ()
        }
    }

    # Parse JSON into typed structure
    data = match json.decode[UsersFile](content) {
        Ok(d) -> d,
        Err(e) -> {
            println("Failed to parse JSON: " ++ e)
            return ()
        }
    }

    # Process the data
    activeUsers = data.users.filter(u => u.email.contains("@"))

    println("Found " ++ show(activeUsers.length()) ++ " users:")
    activeUsers.map(u => println("  - " ++ u.name ++ " <" ++ u.email ++ ">"))
}
```

**Sample users.json:**
```json
{
  "users": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"}
  ]
}
```

## HTTP Request and Parse Response

```nostos
import http
import json

type Post = { id: Int, title: String, body: String, userId: Int }

# Fetch a single post
fetchPost(id: Int) -> Result[Post, String] = {
    response = http.get("https://jsonplaceholder.typicode.com/posts/" ++ show(id))
    match response {
        Ok(r) -> {
            if r.status == 200 {
                json.decode[Post](r.body)
            } else {
                Err("HTTP " ++ show(r.status))
            }
        },
        Err(e) -> Err(e)
    }
}

# Fetch multiple posts concurrently
fetchPosts(ids: List[Int]) -> List[Result[Post, String]] = {
    ids.parMap(id => fetchPost(id))
}

main() = {
    # Single request
    match fetchPost(1) {
        Ok(post) -> println("Title: " ++ post.title),
        Err(e) -> println("Error: " ++ e)
    }

    # Multiple concurrent requests
    results = fetchPosts([1, 2, 3, 4, 5])
    successes = results.filter(r => r.isOk()).map(r => r.unwrap())
    println("Fetched " ++ show(successes.length()) ++ " posts")
}
```

## POST Request with JSON Body

```nostos
import http
import json

type CreateUser = { name: String, email: String }
type UserResponse = { id: Int, name: String, email: String }

createUser(user: CreateUser) -> Result[UserResponse, String] = {
    body = json.encode(user)
    response = http.post(
        "https://api.example.com/users",
        body: body,
        headers: %{
            "Content-Type": "application/json",
            "Authorization": "Bearer " ++ getApiKey()
        }
    )
    match response {
        Ok(r) -> {
            if r.status >= 200 && r.status < 300 {
                json.decode[UserResponse](r.body)
            } else {
                Err("HTTP " ++ show(r.status) ++ ": " ++ r.body)
            }
        },
        Err(e) -> Err(e)
    }
}

getApiKey() = env("API_KEY").unwrapOr("default-key")

main() = {
    newUser = CreateUser(name: "Alice", email: "alice@example.com")
    match createUser(newUser) {
        Ok(created) -> println("Created user with ID: " ++ show(created.id)),
        Err(e) -> println("Failed: " ++ e)
    }
}
```

## Simple CLI Tool

```nostos
import file
import env

# Word count tool: count lines, words, chars in files

countFile(path: String) -> Result[(Int, Int, Int), String] = {
    match file.read(path) {
        Ok(content) -> {
            lines = content.split("\n").length()
            words = content.split(" ").flatMap(s => s.split("\n")).filter(s => s.length() > 0).length()
            chars = content.length()
            Ok((lines, words, chars))
        },
        Err(e) -> Err(e)
    }
}

printUsage() = {
    println("Usage: wc <file1> [file2] ...")
    println("Count lines, words, and characters in files")
}

main() = {
    args = env.args()  # Get command line arguments

    if args.length() < 2 {
        printUsage()
        return ()
    }

    files = args.drop(1)  # Skip program name

    var totalLines = 0
    var totalWords = 0
    var totalChars = 0

    files.map(path => {
        match countFile(path) {
            Ok((lines, words, chars)) -> {
                println(show(lines) ++ "\t" ++ show(words) ++ "\t" ++ show(chars) ++ "\t" ++ path)
                totalLines = totalLines + lines
                totalWords = totalWords + words
                totalChars = totalChars + chars
            },
            Err(e) -> println("Error reading " ++ path ++ ": " ++ e)
        }
    })

    if files.length() > 1 {
        println(show(totalLines) ++ "\t" ++ show(totalWords) ++ "\t" ++ show(totalChars) ++ "\ttotal")
    } else {
        ()
    }
}
```

## Simple HTTP Server

```nostos
import http.server
import json

type Todo = { id: Int, title: String, done: Bool }

mvar todos: List[Todo] = [
    Todo(1, "Learn Nostos", false),
    Todo(2, "Build something", false)
]

mvar nextId: Int = 3

handleRequest(req: Request) -> Response = {
    match (req.method, req.path) {
        ("GET", "/todos") -> {
            Response(
                status: 200,
                body: json.encode(todos),
                headers: %{"Content-Type": "application/json"}
            )
        },

        ("POST", "/todos") -> {
            match json.decode[{title: String}](req.body) {
                Ok(data) -> {
                    newTodo = Todo(nextId, data.title, false)
                    nextId = nextId + 1
                    todos = todos.push(newTodo)
                    Response(
                        status: 201,
                        body: json.encode(newTodo),
                        headers: %{"Content-Type": "application/json"}
                    )
                },
                Err(e) -> Response(status: 400, body: "Invalid JSON: " ++ e)
            }
        },

        ("DELETE", path) -> {
            if path.startsWith("/todos/") {
                idStr = path.substring(7, path.length())
                match idStr.parseInt() {
                    Some(id) -> {
                        todos = todos.filter(t => t.id != id)
                        Response(status: 204, body: "")
                    },
                    None -> Response(status: 400, body: "Invalid ID")
                }
            } else {
                Response(status: 404, body: "Not found")
            }
        },

        _ -> Response(status: 404, body: "Not found")
    }
}

main() = {
    println("Server running on http://localhost:8080")
    http.server.run(8080, handleRequest)
}
```

## Data Pipeline with Error Handling

```nostos
import file
import json

type RawRecord = { id: String, value: String, timestamp: String }
type ProcessedRecord = { id: Int, value: Float, timestamp: Int }

# Parse a raw record, returning errors for invalid data
parseRecord(raw: RawRecord) -> Result[ProcessedRecord, String] = {
    id = match raw.id.parseInt() {
        Some(n) -> n,
        None -> return Err("Invalid id: " ++ raw.id)
    }

    value = match raw.value.parseFloat() {
        Some(f) -> f,
        None -> return Err("Invalid value: " ++ raw.value)
    }

    timestamp = match raw.timestamp.parseInt() {
        Some(t) -> t,
        None -> return Err("Invalid timestamp: " ++ raw.timestamp)
    }

    Ok(ProcessedRecord(id, value, timestamp))
}

# Process a file, collecting successes and errors
processFile(path: String) -> (List[ProcessedRecord], List[String]) = {
    content = file.read(path).unwrapOr("[]")
    records = json.decode[List[RawRecord]](content).unwrapOr([])

    results = records.map(r => parseRecord(r))

    successes = results.filter(r => r.isOk()).map(r => r.unwrap())
    errors = results.filter(r => r.isErr()).map(r => r.unwrapErr())

    (successes, errors)
}

main() = {
    (records, errors) = processFile("data.json")

    println("Processed " ++ show(records.length()) ++ " records")

    if errors.nonEmpty() {
        println("Errors:")
        errors.map(e => println("  - " ++ e))
    } else {
        ()
    }

    # Calculate statistics
    if records.nonEmpty() {
        values = records.map(r => r.value)
        avg = values.sum() / values.length().toFloat()
        println("Average value: " ++ show(avg))
    } else {
        ()
    }
}
```

## WebSocket Client

```nostos
import websocket
import json

type Message = { type: String, content: String }

main() = {
    ws = websocket.connect("wss://echo.websocket.org")

    # Send a message
    msg = Message(type: "greeting", content: "Hello!")
    ws.send(json.encode(msg))

    # Receive response
    response = ws.receive()
    println("Received: " ++ response)

    # Close connection
    ws.close()
}
```

## Concurrent Task Processing

```nostos
import concurrent

type Task = { id: Int, data: String }
type TaskResult = { id: Int, result: String, success: Bool }

# Simulate processing a task
processTask(task: Task) -> TaskResult = {
    # Simulate work
    sleep(100)

    if task.data.length() > 0 {
        TaskResult(task.id, "Processed: " ++ task.data.toUpper(), true)
    } else {
        TaskResult(task.id, "Empty data", false)
    }
}

main() = {
    tasks = [
        Task(1, "hello"),
        Task(2, "world"),
        Task(3, ""),
        Task(4, "nostos"),
        Task(5, "rocks")
    ]

    # Process all tasks concurrently
    println("Processing " ++ show(tasks.length()) ++ " tasks...")

    results = tasks.parMap(t => processTask(t))

    successes = results.filter(r => r.success)
    failures = results.filter(r => !r.success)

    println("Completed: " ++ show(successes.length()) ++ " succeeded, " ++
            show(failures.length()) ++ " failed")

    successes.map(r => println("  [" ++ show(r.id) ++ "] " ++ r.result))
}
```

## Configuration with Defaults

```nostos
import file
import json
import env

type Config = {
    host: String,
    port: Int,
    debug: Bool,
    maxConnections: Int
}

defaultConfig = Config(
    host: "localhost",
    port: 8080,
    debug: false,
    maxConnections: 100
)

# Load config from file, falling back to defaults
loadConfig(path: String) -> Config = {
    # Try file first
    fileConfig = match file.read(path) {
        Ok(content) -> json.decode[Config](content).ok(),
        Err(_) -> None
    }

    match fileConfig {
        Some(c) -> c,
        None -> {
            # Fall back to environment variables
            Config(
                host: env("HOST").unwrapOr(defaultConfig.host),
                port: env("PORT").flatMap(s => s.parseInt()).unwrapOr(defaultConfig.port),
                debug: env("DEBUG").map(s => s == "true").unwrapOr(defaultConfig.debug),
                maxConnections: env("MAX_CONN").flatMap(s => s.parseInt()).unwrapOr(defaultConfig.maxConnections)
            )
        }
    }
}

main() = {
    config = loadConfig("config.json")

    println("Starting server with config:")
    println("  Host: " ++ config.host)
    println("  Port: " ++ show(config.port))
    println("  Debug: " ++ show(config.debug))
    println("  Max connections: " ++ show(config.maxConnections))
}
```
