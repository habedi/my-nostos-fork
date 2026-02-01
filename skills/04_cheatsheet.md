# Nostos Quick Reference Cheat Sheet

One-page syntax and patterns reference.

## Basics

```nostos
# Comments start with hash
x = 42              # Immutable binding
var y = 0           # Mutable local variable
mvar z: Int = 0     # Mutable module-level variable
const PI = 3.14     # Compile-time constant
```

## Types

```nostos
Int Float Bool String Char ()     # Primitives
List[T] Map[K,V] Set[T]           # Collections
Option[T] Result[T,E]             # Common patterns
(A, B, C)                         # Tuples

type Point = { x: Int, y: Int }   # Record
type Color = Red | Green | Blue   # Variants
type Tree[T] = Leaf(T) | Node(Tree[T], Tree[T])
```

## Literals

```nostos
42  -17  1_000_000  0xFF  0b1010  # Int
3.14  -0.5  1.0e10                # Float
true  false                       # Bool
"hello"  'c'  '\n'                # String, Char
[1, 2, 3]                         # List
%{"a": 1, "b": 2}                 # Map
(1, "two", true)                  # Tuple
```

## Operators

```nostos
+  -  *  /  %                     # Arithmetic
==  !=  <  <=  >  >=              # Comparison
&&  ||  !                         # Logical
++                                # String concat
|>                                # Pipe (x |> f  =  f(x))
```

## Functions

```nostos
# Definition
add(a: Int, b: Int) -> Int = a + b
greet(name) = "Hello, " ++ name   # Types inferred

# Lambdas
x => x * 2
(a, b) => a + b

# Call
add(1, 2)
1.add(2)                          # UFCS

# Pattern matching in definition
factorial(0) = 1
factorial(n) = n * factorial(n - 1)
```

## Control Flow

```nostos
# If expression
if x > 0 { "positive" } else { "non-positive" }
if x > 0 then "positive" else "non-positive"

# Match
match value {
    0 -> "zero",
    n if n > 0 -> "positive",
    _ -> "negative"
}

# Loops
while condition { body }
for item in list { body }
(1..10).map(i => process(i))      # Prefer this
```

## Pattern Matching

```nostos
# Literals
match x { 1 -> "one", 2 -> "two", _ -> "other" }

# Destructuring
(a, b) = tuple
{ name, age } = person
[h | t] = list                    # Head and tail
[a, b, c] = threeList             # Exact length

# Variants
match opt {
    Some(x) -> use(x),
    None -> default
}

# Guards
match n {
    x if x > 0 -> "positive",
    x if x < 0 -> "negative",
    _ -> "zero"
}
```

## Collections

```nostos
# List
[1, 2, 3].map(x => x * 2)         # [2, 4, 6]
[1, 2, 3].filter(x => x > 1)      # [2, 3]
[1, 2, 3].fold(0, (a,b) => a+b)   # 6
list.length()  list[0]  list.get(0)

# Map
map = %{"a": 1, "b": 2}
map["a"]  map.get("a")
map.insert("c", 3)
map.keys()  map.values()

# Set
Set.from([1, 2, 2, 3])            # {1, 2, 3}
set.contains(x)  set.insert(x)
a.union(b)  a.intersection(b)
```

## Option & Result

```nostos
# Option
Some(42)  None
opt.map(f)  opt.flatMap(f)
opt.unwrap()  opt.unwrapOr(default)
opt.isSome()  opt.isNone()

# Result
Ok(value)  Err(error)
res.map(f)  res.mapErr(f)
res.unwrap()  res.unwrapOr(default)
res.isOk()  res.isErr()
```

## Error Handling

```nostos
# Try/catch
try {
    riskyOperation()
} catch {
    "specific" -> handleSpecific(),
    e -> handleGeneric(e)
}

# Throw
throw("error message")

# Early return
if bad { return Err("failed") } else { () }
```

## Modules

```nostos
# math.nos
pub add(a, b) = a + b             # Public
helper(x) = x * 2                 # Private

# main.nos
import math
math.add(1, 2)
```

## Traits

```nostos
trait Show {
    show(self) -> String
}

impl Show for Point {
    show(self) = "(" ++ show(self.x) ++ "," ++ show(self.y) ++ ")"
}
```

## Templates

```nostos
template logged(fn) = quote {
    println(">>> " ++ ~fn.name)
    ~fn.body
}

@logged
compute() = 42

# Key operations
quote { code }                    # Capture AST
~expr                             # Splice AST
eval("code string")               # Parse string as code
gensym("prefix")                  # Unique identifier
~fn.name  ~fn.body  ~fn.params    # Function metadata
~typeDef.name  ~typeDef.fields    # Type metadata
```

## Concurrency

```nostos
import concurrent

handle = spawn(() => computation())
result = await(handle)
sleep(1000)                       # Milliseconds
list.parMap(f)                    # Parallel map
```

## I/O

```nostos
# Print
print("no newline")
println("with newline")
show(42)                          # Any to String

# Files
import file
file.read("path")                 # Result[String, String]
file.write("path", content)       # Result[(), String]

# HTTP
import http
http.get(url)
http.post(url, body: data, headers: %{...})

# JSON
import json
json.parse(str)                   # Result[Json, String]
json.decode[Type](str)            # Result[Type, String]
json.encode(value)                # String
```

## Common Patterns

```nostos
# Pipeline
data
    .filter(x => x.valid)
    .map(x => transform(x))
    .fold(init, combine)

# Safe unwrap with default
value = opt.unwrapOr(default)

# Error propagation
match operation() {
    Ok(x) -> continue(x),
    Err(e) -> return Err(e)
}

# Builder pattern
Config()
    .withHost("localhost")
    .withPort(8080)
    .build()
```

## Comparison to Other Languages

| Nostos | Python | Rust | JavaScript |
|--------|--------|------|------------|
| `#` comment | `#` comment | `//` comment | `//` comment |
| `"a" ++ "b"` | `"a" + "b"` | `format!()` | `"a" + "b"` |
| `[1,2,3]` | `[1,2,3]` | `vec![1,2,3]` | `[1,2,3]` |
| `%{"a":1}` | `{"a":1}` | `HashMap` | `{a:1}` |
| `x => x*2` | `lambda x: x*2` | `\|x\| x*2` | `x => x*2` |
| `List[Int]` | `list[int]` | `Vec<i32>` | N/A |
| `Some(x)` | `x` or `None` | `Some(x)` | `x` or `null` |
| `match {}` | `match` (3.10+) | `match {}` | `switch` |
