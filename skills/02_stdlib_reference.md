# Nostos Standard Library Reference

Quick reference for methods on built-in types.

## String

```nostos
s = "Hello, World!"

# Length and access
s.length()              # 13
s.charAt(0)             # 'H'
s.substring(0, 5)       # "Hello"

# Case conversion
s.toLower()             # "hello, world!"
s.toUpper()             # "HELLO, WORLD!"

# Search
s.contains("World")     # true
s.startsWith("Hello")   # true
s.endsWith("!")         # true
s.indexOf("o")          # Some(4)
s.lastIndexOf("o")      # Some(8)

# Manipulation
s.trim()                # Remove whitespace from ends
s.trimStart()           # Remove leading whitespace
s.trimEnd()             # Remove trailing whitespace
s.replace("World", "Nostos")  # "Hello, Nostos!"
s.replaceAll("l", "L")  # "HeLLo, WorLd!"

# Split and join
"a,b,c".split(",")      # ["a", "b", "c"]
["a", "b", "c"].join("-")  # "a-b-c"

# Conversion
"42".parseInt()         # Some(42) or None
"3.14".parseFloat()     # Some(3.14) or None
show(42)                # "42" (any value to string)

# Concatenation
"Hello" ++ " " ++ "World"  # "Hello World"

# Characters
s.chars()               # List of Char
s.bytes()               # List of Int (UTF-8 bytes)
```

## List[T]

```nostos
list = [1, 2, 3, 4, 5]

# Length and access
list.length()           # 5
list[0]                 # 1 (panics if out of bounds)
list.get(0)             # Some(1) or None
list.first()            # Some(1) or None
list.last()             # Some(5) or None

# Add/remove
list.push(6)            # [1, 2, 3, 4, 5, 6]
list.append([6, 7])     # [1, 2, 3, 4, 5, 6, 7]
list.prepend(0)         # [0, 1, 2, 3, 4, 5]
[h | t] = list          # h = 1, t = [2, 3, 4, 5]

# Transform
list.map(x => x * 2)    # [2, 4, 6, 8, 10]
list.filter(x => x > 2) # [3, 4, 5]
list.flatMap(x => [x, x])  # [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]

# Reduce
list.fold(0, (acc, x) => acc + x)  # 15
list.reduce((a, b) => a + b)       # 15 (no initial value)
list.sum()              # 15 (for numeric lists)
list.product()          # 120 (for numeric lists)

# Search
list.find(x => x > 3)   # Some(4)
list.any(x => x > 3)    # true
list.all(x => x > 0)    # true
list.contains(3)        # true
list.indexOf(3)         # Some(2)

# Sort and reverse
list.sort()             # [1, 2, 3, 4, 5]
list.sortBy(x => -x)    # [5, 4, 3, 2, 1]
list.reverse()          # [5, 4, 3, 2, 1]

# Slice
list.take(3)            # [1, 2, 3]
list.drop(2)            # [3, 4, 5]
list.slice(1, 4)        # [2, 3, 4]
list.takeWhile(x => x < 4)  # [1, 2, 3]
list.dropWhile(x => x < 3)  # [3, 4, 5]

# Combine
list.zip([10, 20, 30])  # [(1, 10), (2, 20), (3, 30)]
[[1, 2], [3, 4]].flatten()  # [1, 2, 3, 4]

# Check
list.isEmpty()          # false
list.nonEmpty()         # true

# Convert
list.toSet()            # Set with unique elements
```

## Map[K, V]

```nostos
map = %{"a": 1, "b": 2, "c": 3}

# Access
map["a"]                # 1 (panics if missing)
map.get("a")            # Some(1) or None
map.getOrElse("x", 0)   # 0 (default if missing)

# Modify (returns new map)
map.insert("d", 4)      # %{"a": 1, "b": 2, "c": 3, "d": 4}
map.remove("a")         # %{"b": 2, "c": 3}
map.update("a", v => v + 10)  # %{"a": 11, "b": 2, "c": 3}

# Query
map.contains("a")       # true
map.size()              # 3
map.isEmpty()           # false

# Iterate
map.keys()              # ["a", "b", "c"]
map.values()            # [1, 2, 3]
map.entries()           # [("a", 1), ("b", 2), ("c", 3)]

# Transform
map.map((k, v) => (k, v * 2))     # %{"a": 2, "b": 4, "c": 6}
map.filter((k, v) => v > 1)       # %{"b": 2, "c": 3}
map.mapValues(v => v * 2)         # %{"a": 2, "b": 4, "c": 6}

# Merge
map.merge(%{"c": 30, "d": 4})     # %{"a": 1, "b": 2, "c": 30, "d": 4}
```

## Set[T]

```nostos
set = Set.from([1, 2, 3, 2, 1])  # {1, 2, 3}

# Modify (returns new set)
set.insert(4)           # {1, 2, 3, 4}
set.remove(2)           # {1, 3}

# Query
set.contains(2)         # true
set.size()              # 3
set.isEmpty()           # false

# Set operations
a = Set.from([1, 2, 3])
b = Set.from([2, 3, 4])
a.union(b)              # {1, 2, 3, 4}
a.intersection(b)       # {2, 3}
a.difference(b)         # {1}
a.isSubsetOf(b)         # false

# Convert
set.toList()            # [1, 2, 3]
```

## Option[T]

```nostos
some = Some(42)
none = None

# Check
some.isSome()           # true
none.isNone()           # true

# Extract
some.unwrap()           # 42 (panics if None)
none.unwrapOr(0)        # 0
some.getOrElse(0)       # 42

# Transform
some.map(x => x * 2)    # Some(84)
none.map(x => x * 2)    # None
some.flatMap(x => Some(x + 1))  # Some(43)
some.filter(x => x > 50)        # None

# Convert
some.toList()           # [42]
none.toList()           # []
some.okOr("error")      # Ok(42)
none.okOr("error")      # Err("error")

# Pattern match
match opt {
    Some(x) -> "got " ++ show(x),
    None -> "nothing"
}
```

## Result[T, E]

```nostos
ok = Ok(42)
err = Err("failed")

# Check
ok.isOk()               # true
err.isErr()             # true

# Extract
ok.unwrap()             # 42 (panics if Err)
err.unwrapErr()         # "failed"
ok.unwrapOr(0)          # 42
err.unwrapOr(0)         # 0

# Transform
ok.map(x => x * 2)      # Ok(84)
err.map(x => x * 2)     # Err("failed")
ok.mapErr(e => "Error: " ++ e)   # Ok(42)
err.mapErr(e => "Error: " ++ e)  # Err("Error: failed")
ok.flatMap(x => Ok(x + 1))       # Ok(43)

# Convert
ok.ok()                 # Some(42)
err.ok()                # None
ok.err()                # None
err.err()               # Some("failed")

# Pattern match
match result {
    Ok(value) -> "success: " ++ show(value),
    Err(e) -> "error: " ++ e
}
```

## Int / Float

```nostos
# Int operations
42.abs()                # 42
(-42).abs()             # 42
10.max(20)              # 20
10.min(20)              # 10
10.clamp(5, 15)         # 10
2.pow(10)               # 1024

# Float operations
3.14.floor()            # 3.0
3.14.ceil()             # 4.0
3.14.round()            # 3.0
3.14.abs()              # 3.14
(-1.5).abs()            # 1.5
4.0.sqrt()              # 2.0
2.0.pow(3.0)            # 8.0

# Conversion
42.toFloat()            # 42.0
3.14.toInt()            # 3
42.toString()           # "42"

# Ranges
(1..5)                  # [1, 2, 3, 4] (exclusive end)
(1..=5)                 # [1, 2, 3, 4, 5] (inclusive end)
```

## Char

```nostos
c = 'A'

c.isAlpha()             # true
c.isDigit()             # false
c.isAlphanumeric()      # true
c.isWhitespace()        # false
c.isUpper()             # true
c.isLower()             # false
c.toLower()             # 'a'
c.toUpper()             # 'A'
c.toInt()               # 65 (ASCII/Unicode value)
Char.fromInt(65)        # 'A'
```

## Tuple

```nostos
t = (1, "hello", true)

# Access by destructuring
(a, b, c) = t           # a = 1, b = "hello", c = true

# Two-element tuples have .0 and .1
pair = (10, 20)
pair.0                  # 10
pair.1                  # 20

# Swap
(10, 20).swap()         # (20, 10)
```

## Common Utility Functions

```nostos
# Printing
print("no newline")
println("with newline")
println("Value: " ++ show(42))

# Assertions (panic on failure)
assert_eq(expected, actual)
assert_eq(4, 2 + 2)

# Type conversion
show(anyValue)          # Convert to String
"42".parseInt()         # String to Int
"3.14".parseFloat()     # String to Float

# Comparison
min(a, b)               # Smaller of two
max(a, b)               # Larger of two

# Ranges
range(0, 5)             # [0, 1, 2, 3, 4]
range(0, 10, 2)         # [0, 2, 4, 6, 8] (with step)
```

## File I/O

```nostos
import file

# Read
content = file.read("path.txt")           # Result[String, String]
lines = file.readLines("path.txt")        # Result[List[String], String]
bytes = file.readBytes("path.bin")        # Result[List[Int], String]

# Write
file.write("path.txt", "content")         # Result[(), String]
file.writeLines("path.txt", ["a", "b"])   # Result[(), String]
file.append("path.txt", "more")           # Result[(), String]

# Check
file.exists("path.txt")                   # Bool
file.isFile("path.txt")                   # Bool
file.isDir("path")                        # Bool

# Directory
file.listDir(".")                         # Result[List[String], String]
file.createDir("newdir")                  # Result[(), String]
```

## JSON

```nostos
import json

# Parse
json.parse('{"name": "Alice", "age": 30}')  # Result[Json, String]

# Access Json values
j = json.parse('{"items": [1, 2, 3]}').unwrap()
j["items"]              # Json array
j["items"][0]           # Json number

# Convert to typed value
type Person = { name: String, age: Int }
json.decode[Person]('{"name": "Alice", "age": 30}')  # Result[Person, String]

# Encode
json.encode(Person("Alice", 30))  # '{"name":"Alice","age":30}'
```

## HTTP

```nostos
import http

# GET request
response = http.get("https://api.example.com/data")  # Result[Response, String]
body = response.unwrap().body                        # String

# POST with JSON
response = http.post("https://api.example.com/users",
    body: '{"name": "Alice"}',
    headers: %{"Content-Type": "application/json"}
)

# Response fields
response.status         # Int (200, 404, etc.)
response.body           # String
response.headers        # Map[String, String]
```

## Concurrency

```nostos
import concurrent

# Spawn task
handle = spawn(() => expensiveComputation())

# Wait for result
result = await(handle)

# Sleep
sleep(1000)             # Milliseconds

# Parallel map
results = [1, 2, 3].parMap(x => compute(x))

# Channels
ch = channel()
spawn(() => ch.send(42))
value = ch.receive()    # 42
```
