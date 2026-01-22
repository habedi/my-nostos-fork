# Functions in Nostos

## Basic Function Definition

```nostos
# Single expression (no braces needed)
add(a: Int, b: Int) -> Int = a + b

# Multi-statement with block
greet(name: String) -> String = {
    prefix = "Hello, "
    prefix ++ name ++ "!"
}

# Type inference (return type can be omitted)
double(x) = x * 2

# Unit return (for side effects)
logMessage(msg: String) = println(msg)
```

## Calling Functions

```nostos
result = add(2, 3)          # 5
message = greet("Alice")    # "Hello, Alice!"

# UFCS (Uniform Function Call Syntax)
# First argument can be receiver
5.double()                  # Same as double(5)
"Alice".greet()             # Same as greet("Alice")
```

## Named and Default Parameters

```nostos
# Named parameters
connect(host: String, port: Int, timeout: Int) =
    println("Connecting to " ++ host ++ ":" ++ show(port))

# Call with named arguments
connect(host: "localhost", port: 8080, timeout: 30)

# Default values
greet(name: String, greeting: String = "Hello") =
    greeting ++ ", " ++ name ++ "!"

greet("Alice")              # "Hello, Alice!"
greet("Alice", "Hi")        # "Hi, Alice!"
```

## Closures (Anonymous Functions)

```nostos
# Lambda syntax
double = x => x * 2
add = (a, b) => a + b

# Multi-statement closure
process = x => {
    y = x * 2
    y + 1
}

# Used with higher-order functions
[1, 2, 3].map(x => x * 2)           # [2, 4, 6]
[1, 2, 3].filter(x => x > 1)        # [2, 3]
[1, 2, 3].fold(0, (acc, x) => acc + x)  # 6
```

## Closures Capturing Variables

```nostos
makeCounter() = {
    var count = 0
    () => {
        count = count + 1
        count
    }
}

counter = makeCounter()
counter()   # 1
counter()   # 2
counter()   # 3
```

## Recursion

```nostos
# Simple recursion
factorial(0) = 1
factorial(n) = n * factorial(n - 1)

# Tail recursion (optimized)
factorialTail(n) = go(n, 1)
go(0, acc) = acc
go(n, acc) = go(n - 1, n * acc)

# Recursive list processing
sum([]) = 0
sum([h | t]) = h + sum(t)
```

## Generic Functions

```nostos
# Type parameter in brackets
identity[T](x: T) -> T = x

# Multiple type parameters
pair[A, B](a: A, b: B) -> (A, B) = (a, b)

# With trait bounds
printAll[T: Show](items: List[T]) =
    items.map(x => x.show()).join(", ")
```

## Early Return

```nostos
findFirst(items: List[Int], target: Int) -> Int = {
    var i = 0
    while i < items.length() {
        if items[i] == target then
            return i
        else
            ()
        i = i + 1
    }
    -1  # Not found
}
```

## Function Composition

```nostos
# Compose functions
double(x) = x * 2
addOne(x) = x + 1

# Manual composition
composed(x) = addOne(double(x))

# Using pipe-style with UFCS
result = 5.double().addOne()    # 11

# Method chaining
[1, 2, 3]
    .map(x => x * 2)
    .filter(x => x > 2)
    .fold(0, (a, b) => a + b)
```

## Higher-Order Functions

```nostos
# Function taking a function
applyTwice(f, x) = f(f(x))

applyTwice(x => x * 2, 3)   # 12

# Function returning a function
multiplier(n) = x => x * n

triple = multiplier(3)
triple(4)   # 12

# Common patterns
[1, 2, 3].map(x => x * 2)           # Transform each
[1, 2, 3].filter(x => x > 1)        # Keep matching
[1, 2, 3].fold(0, (a, b) => a + b)  # Reduce to one
[1, 2, 3].any(x => x > 2)           # true if any match
[1, 2, 3].all(x => x > 0)           # true if all match
[1, 2, 3].find(x => x > 1)          # Some(2)
```
