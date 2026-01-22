# Nostos Basics

## Comments

```nostos
# This is a single-line comment
# Comments use the hash symbol
```

## Literals

```nostos
# Integers
42
-17
1_000_000       # Underscores for readability
0xFF            # Hexadecimal
0b1010          # Binary

# Floats
3.14
-0.5
1.0e10          # Scientific notation

# Booleans
true
false

# Strings (double or single quotes)
"Hello, World!"
'{"key": "value"}'   # Useful for JSON

# Characters
'a'
'\n'            # Newline
'\t'            # Tab

# Unit (empty value, like void)
()
```

## Variables

```nostos
# Immutable binding (default)
x = 42
name = "Alice"

# Mutable variable (use sparingly)
var counter = 0
counter = counter + 1

# Type annotations (optional, inferred)
x: Int = 42
name: String = "Alice"
```

## Basic Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit integer | `42` |
| `Float` | 64-bit float | `3.14` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `Char` | Single character | `'a'` |
| `()` | Unit type | `()` |

## Operators

```nostos
# Arithmetic
1 + 2       # Addition
5 - 3       # Subtraction
4 * 2       # Multiplication
10 / 3      # Integer division
10 % 3      # Modulo

# Comparison
x == y      # Equal
x != y      # Not equal
x < y       # Less than
x <= y      # Less or equal
x > y       # Greater than
x >= y      # Greater or equal

# Logical
a && b      # And
a || b      # Or
!a          # Not

# String concatenation
"Hello" ++ " " ++ "World"   # "Hello World"
```

## Printing

```nostos
print("No newline")
println("With newline")

# Convert to string with show()
println("Value: " ++ show(42))
```

## Hello World

```nostos
# Every program needs a main() function
main() = {
    println("Hello, World!")
}

# Or single expression
main() = println("Hello, World!")
```

## Constants

```nostos
# Module-level constants (evaluated at compile time)
const PI = 3.14159
const MAX_SIZE = 1000
const GREETING = "Hello"

main() = println(GREETING ++ ", PI is " ++ show(PI))
```

## Assertions

```nostos
# assert_eq checks equality, panics on failure
assert_eq(4, 2 + 2)
assert_eq("hello", "hel" ++ "lo")

# Useful in tests
main() = {
    assert_eq(42, 6 * 7)
    println("All assertions passed!")
}
```
