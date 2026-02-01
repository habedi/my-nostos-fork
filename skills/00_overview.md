# Nostos Language Overview

Nostos is a modern functional programming language with a focus on expressiveness, safety, and metaprogramming.

## Key Characteristics

**Expression-based**: Everything is an expression that returns a value. No statements.
```nostos
# if/else returns a value
result = if x > 0 { "positive" } else { "non-positive" }

# Blocks return their last expression
value = {
    a = 10
    b = 20
    a + b  # This is returned
}
```

**Algebraic Data Types**: First-class sum types (variants) and product types (records).
```nostos
type Result[T, E] = Ok(T) | Err(E)
type Person = { name: String, age: Int }
```

**Pattern Matching**: Destructure data elegantly.
```nostos
match result {
    Ok(value) -> "Got: " ++ show(value),
    Err(msg) -> "Error: " ++ msg
}
```

**UFCS (Uniform Function Call Syntax)**: Call any function as a method.
```nostos
# These are equivalent:
double(5)
5.double()

# Enables fluent chaining:
[1, 2, 3].map(x => x * 2).filter(x => x > 2).sum()
```

**Compile-time Metaprogramming**: Templates generate code at compile time.
```nostos
template logged(fn) = quote {
    println("Calling " ++ ~fn.name)
    ~fn.body
}

@logged
compute() = 42  # Prints "Calling compute" when called
```

**Hindley-Milner Type Inference**: Types are inferred but can be annotated.
```nostos
# Compiler infers: add(Int, Int) -> Int
add(a, b) = a + b

# Explicit when needed
identity[T](x: T) -> T = x
```

## Compared to Other Languages

| Feature | Nostos | Similar to |
|---------|--------|------------|
| Algebraic types | `type Option[T] = Some(T) \| None` | Rust, Haskell, OCaml |
| Pattern matching | `match x { ... }` | Rust, Scala, Elixir |
| Lambdas | `x => x * 2` | JavaScript, Kotlin |
| String concat | `"a" ++ "b"` | Haskell, Elixir |
| List literals | `[1, 2, 3]` | Python, JavaScript |
| Map literals | `%{"a": 1}` | Elixir (similar) |
| Immutable by default | `x = 5` | Rust, Scala |
| UFCS | `5.double()` | D, Nim |
| Templates/macros | `@decorator` | Rust (proc macros), Elixir |
| Traits | `trait Show { show() }` | Rust, Scala, Haskell (typeclasses) |

## What Nostos is Good For

- **Scripting and automation** - Concise syntax, fast startup
- **Data transformation** - Pattern matching, list operations
- **Web services** - Built-in HTTP server/client, JSON support
- **DSLs** - Templates enable domain-specific abstractions
- **Learning FP** - Clean syntax without ceremony

## Program Structure

```nostos
# Imports (optional)
import json
import http

# Type definitions
type User = { id: Int, name: String }

# Constants
const API_URL = "https://api.example.com"

# Functions
fetchUser(id: Int) -> Result[User, String] = {
    # ...
}

# Entry point
main() = {
    match fetchUser(1) {
        Ok(user) -> println("Hello, " ++ user.name),
        Err(e) -> println("Error: " ++ e)
    }
}
```

## Module System

```nostos
# math.nos
pub add(a: Int, b: Int) = a + b      # Public
helper(x: Int) = x * 2               # Private (no pub)

# main.nos
import math
main() = math.add(1, 2)              # Use qualified name
```

## Concurrency Model

Nostos uses lightweight tasks (green threads) with message passing:

```nostos
import concurrent

main() = {
    # Spawn concurrent task
    handle = spawn(() => {
        sleep(100)
        42
    })

    # Wait for result
    result = await(handle)  # 42
}
```

## Error Handling

Two mechanisms: `Result` types for expected errors, exceptions for unexpected ones.

```nostos
# Result for expected failures
parseNumber(s: String) -> Result[Int, String] = {
    # ...
}

# Exceptions for unexpected failures
riskyOperation() = {
    if somethingWrong {
        throw("Unexpected error")
    }
    result
}

# Catch exceptions
safe() = try {
    riskyOperation()
} catch {
    e -> "Caught: " ++ e
}
```
