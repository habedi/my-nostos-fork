# File I/O in Nostos

## Reading Files

```nostos
# Read entire file as string
content = File.readAll("config.txt")
println(content)

# Read file as lines
lines = File.readLines("data.txt")
lines.each(line => println(line))

# Read with error handling
content = try {
    File.readAll("maybe-missing.txt")
} catch e {
    println("File error: " ++ e)
    ""  # Default empty string
}
```

## Writing Files

```nostos
# Write string to file (creates or overwrites)
File.writeAll("output.txt", "Hello, World!")

# Write lines
lines = ["Line 1", "Line 2", "Line 3"]
File.writeAll("output.txt", lines.join("\n"))

# Append to file
File.append("log.txt", "New log entry\n")
```

## File Handles (Streaming)

```nostos
# Open file for reading
handle = File.open("large.txt", "r")

# Read line by line
line = File.readLine(handle)
while line != "" {
    processLine(line)
    line = File.readLine(handle)
}

# Close when done
File.close(handle)

# Open modes: "r" (read), "w" (write), "a" (append)
```

## Safe File Operations

```nostos
# Read with automatic cleanup
withFile(path: String, mode: String, fn: Handle -> T) -> T = {
    handle = File.open(path, mode)
    try {
        fn(handle)
    } finally {
        File.close(handle)
    }
}

# Usage
result = withFile("data.txt", "r", handle => {
    lines = []
    line = File.readLine(handle)
    while line != "" {
        lines = lines ++ [line]
        line = File.readLine(handle)
    }
    lines
})
```

## File Existence and Info

```nostos
# Check if file exists
if File.exists("config.json") then {
    config = File.readAll("config.json")
} else {
    config = "{}"
}

# Get file size
size = File.size("data.bin")
println("File size: " ++ show(size) ++ " bytes")
```

## Directory Operations

```nostos
# List files in directory
files = File.listDir("./data")
files.each(f => println(f))

# Create directory
File.mkdir("./output")

# Check if path is directory
if File.isDir("./data") then {
    processDirectory("./data")
}
```

## Path Operations

```nostos
# Join paths
fullPath = Path.join("data", "users", "alice.json")
# "data/users/alice.json"

# Get filename from path
name = Path.filename("/home/user/doc.txt")  # "doc.txt"

# Get directory from path
dir = Path.dirname("/home/user/doc.txt")    # "/home/user"

# Get file extension
ext = Path.extension("photo.jpg")           # "jpg"
```

## Binary Files

```nostos
# Read binary data
bytes = File.readBytes("image.png")

# Write binary data
File.writeBytes("output.bin", bytes)

# Work with typed arrays for efficiency
data = Float64Array.from([1.0, 2.0, 3.0])
File.writeBytes("floats.bin", data.toBytes())
```

## CSV Processing

```nostos
# Read CSV
parseCsv(content: String) -> List[List[String]] = {
    content.split("\n")
        .filter(line => line != "")
        .map(line => line.split(","))
}

csv = File.readAll("data.csv")
rows = parseCsv(csv)

# With header
header = rows[0]
data = rows.tail()

# Access by column name
getColumn(rows, name: String) = {
    idx = header.indexOf(name)
    data.map(row => row[idx])
}

names = getColumn(rows, "name")
```

## JSON Configuration Files

```nostos
type Config = {
    host: String,
    port: Int,
    debug: Bool
}

loadConfig(path: String) -> Config = {
    content = File.readAll(path)
    fromJson[Config](content)
}

saveConfig(path: String, config: Config) = {
    content = json.stringifyPretty(toJson(config))
    File.writeAll(path, content)
}

# Usage
config = loadConfig("config.json")
config.port = 9000
saveConfig("config.json", config)
```

## Log File Pattern

```nostos
mvar logHandle: Option[Handle] = None

initLog(path: String) = {
    logHandle = Some(File.open(path, "a"))
}

log(level: String, message: String) = {
    match logHandle {
        Some(handle) -> {
            timestamp = formatTime(currentTimeMillis())
            line = timestamp ++ " [" ++ level ++ "] " ++ message ++ "\n"
            File.write(handle, line)
        }
        None -> ()
    }
}

closeLog() = {
    match logHandle {
        Some(handle) -> {
            File.close(handle)
            logHandle = None
        }
        None -> ()
    }
}

# Usage
initLog("app.log")
log("INFO", "Application started")
log("ERROR", "Something went wrong")
closeLog()
```

## Temporary Files

```nostos
# Create temp file
tempPath = File.tempFile("prefix", ".txt")
File.writeAll(tempPath, "temporary data")

# Use temp file
data = processFile(tempPath)

# Clean up
File.delete(tempPath)
```

## File Watching Pattern

```nostos
watchFile(path: String, onChange: String -> ()) = {
    lastContent = File.readAll(path)

    loop() = {
        sleep(1000)  # Check every second
        currentContent = File.readAll(path)
        if currentContent != lastContent then {
            onChange(currentContent)
            lastContent = currentContent
        }
        loop()
    }
    spawn(loop)
}

# Usage
watchFile("config.json", newContent => {
    println("Config changed!")
    reloadConfig(newContent)
})
```

## Parallel File Processing

```nostos
# Process multiple files in parallel
processFiles(paths: List[String]) -> List[Result] = {
    me = self()

    paths.each((path, i) => spawn(() => {
        result = try {
            content = File.readAll(path)
            Ok(processContent(content))
        } catch e {
            Err(e)
        }
        send(me, (i, result))
    }))

    # Collect in order
    results = paths.map(_ => receive())
    results.sortBy(r => r.0).map(r => r.1)
}
```

## Error Recovery

```nostos
# Retry on transient errors
readWithRetry(path: String, maxRetries: Int) -> Option[String] = {
    attempt(retries) = {
        if retries <= 0 then None
        else {
            try {
                Some(File.readAll(path))
            } catch e {
                println("Read failed: " ++ e ++ ", retrying...")
                sleep(100)
                attempt(retries - 1)
            }
        }
    }
    attempt(maxRetries)
}
```
