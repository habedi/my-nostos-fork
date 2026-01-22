# Strings in Nostos

## String Literals

```nostos
# Double quotes
greeting = "Hello, World!"

# Single quotes (useful for embedded double quotes)
json = '{"name": "Alice", "age": 30}'

# Escape sequences
newline = "Line 1\nLine 2"
tab = "Col1\tCol2"
quote = "She said \"Hello\""
backslash = "C:\\path\\file"
```

## String Concatenation

```nostos
# Use ++ operator
full = "Hello" ++ " " ++ "World"

# Building strings
name = "Alice"
age = 30
message = "Name: " ++ name ++ ", Age: " ++ show(age)

# Multi-part concatenation
result = "Part 1" ++
         " Part 2" ++
         " Part 3"
```

## Converting to String

```nostos
# show() converts any value to string
show(42)           # "42"
show(3.14)         # "3.14"
show(true)         # "true"
show([1, 2, 3])    # "[1, 2, 3]"

# Common pattern
println("Value: " ++ show(someValue))
```

## String Methods

```nostos
s = "Hello, World!"

# Length
s.length()          # 13

# Case conversion
s.toUpper()         # "HELLO, WORLD!"
s.toLower()         # "hello, world!"

# Substring
s.substring(0, 5)   # "Hello"
s.substring(7, 12)  # "World"

# Contains/starts/ends
s.contains("World")     # true
s.startsWith("Hello")   # true
s.endsWith("!")         # true

# Finding
s.indexOf("World")      # 7 (or -1 if not found)

# Trimming whitespace
"  hello  ".trim()      # "hello"
"  hello  ".trimStart() # "hello  "
"  hello  ".trimEnd()   # "  hello"
```

## Split and Join

```nostos
# Split string into list
"a,b,c".split(",")          # ["a", "b", "c"]
"hello world".split(" ")     # ["hello", "world"]

# Join list into string
["a", "b", "c"].join(",")   # "a,b,c"
["hello", "world"].join(" ") # "hello world"

# Useful pattern
csv = "1,2,3,4,5"
numbers = csv.split(",").map(s => parseInt(s))
```

## Replace

```nostos
# Replace first occurrence
"hello world".replace("world", "there")  # "hello there"

# Replace all occurrences
"ababa".replaceAll("a", "x")  # "xbxbx"
```

## Character Access

```nostos
s = "Hello"

# Get character at index
s.charAt(0)         # 'H'
s.charAt(4)         # 'o'

# Get as list of characters
s.chars()           # ['H', 'e', 'l', 'l', 'o']
```

## String Comparison

```nostos
"abc" == "abc"      # true
"abc" != "def"      # true
"abc" < "abd"       # true (lexicographic)
"ABC" < "abc"       # true (uppercase < lowercase)
```

## Parsing Strings

```nostos
# Parse to Int
"42".parseInt()     # 42

# Parse to Float
"3.14".parseFloat() # 3.14

# Safe parsing (returns Result)
"42".tryParseInt()      # Ok(42)
"invalid".tryParseInt() # Err("...")
```

## Multiline Strings

```nostos
# Use regular strings with \n
multiline = "Line 1\nLine 2\nLine 3"

# Or concatenation for readability
poem = "Roses are red,\n" ++
       "Violets are blue,\n" ++
       "Nostos is fun,\n" ++
       "And so are you!"
```

## String Building Pattern

```nostos
# Building a string incrementally
buildReport(items: List[String]) -> String = {
    var result = "Report:\n"
    var i = 0
    while i < items.length() {
        result = result ++ "- " ++ items[i] ++ "\n"
        i = i + 1
    }
    result
}

# Functional style (preferred)
buildReport(items: List[String]) -> String =
    "Report:\n" ++ items.map(s => "- " ++ s).join("\n")
```

## URL Encoding

```nostos
import url

# Encode special characters
url.encode("hello world")   # "hello%20world"
url.encode("a=b&c=d")       # "a%3Db%26c%3Dd"

# Decode
url.decode("hello%20world") # "hello world"
```

## Common Patterns

```nostos
# Check if empty
s.length() == 0
s == ""

# Default for empty
name = if input == "" then "Anonymous" else input

# Format number with padding
padLeft(s: String, len: Int, char: String) -> String = {
    if s.length() >= len then s
    else padLeft(char ++ s, len, char)
}

padLeft(show(42), 5, "0")   # "00042"

# Repeat string
repeat(s: String, n: Int) -> String = {
    if n <= 0 then ""
    else s ++ repeat(s, n - 1)
}

repeat("ab", 3)   # "ababab"
```
