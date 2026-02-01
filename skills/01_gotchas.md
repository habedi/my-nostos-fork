# Nostos Gotchas & Common Mistakes

Things that trip people up when learning Nostos.

## Syntax Differences

### Comments use `#`, not `//`
```nostos
# Correct: hash for comments
// Wrong: this is NOT a comment, it's a syntax error
```

### String concatenation is `++`, not `+`
```nostos
# Correct
"Hello" ++ " " ++ "World"

# Wrong - type error (+ is for numbers)
"Hello" + " " + "World"
```

### No semicolons
```nostos
# Correct - expressions separated by newlines or in blocks
x = 1
y = 2

# Wrong - semicolons are syntax errors
x = 1;
y = 2;
```

### Commas in match arms
```nostos
# Correct - commas between match arms
match x {
    1 -> "one",
    2 -> "two",
    _ -> "other"
}

# Wrong - no commas causes parse error
match x {
    1 -> "one"
    2 -> "two"
}
```

### `then` required in single-line if
```nostos
# Multi-line if (no then needed)
if x > 0 {
    "positive"
} else {
    "non-positive"
}

# Single-line if REQUIRES then
if x > 0 then "positive" else "non-positive"

# Wrong - missing then
if x > 0 "positive" else "non-positive"
```

## Variable Mutability

### `var` vs `mvar` - different scopes
```nostos
# var = local mutable variable (inside functions)
process() = {
    var counter = 0
    counter = counter + 1
    counter
}

# mvar = module-level mutable variable (top-level)
mvar globalCounter: Int = 0

increment() = {
    globalCounter = globalCounter + 1
}

# Wrong - var at module level
var badGlobal = 0  # Error!

# Wrong - mvar inside function
process() = {
    mvar x = 0  # Error!
}
```

### Immutable by default
```nostos
# This creates an immutable binding
x = 42
x = 43  # Error! Cannot reassign

# Use var for mutability
var x = 42
x = 43  # OK
```

## Try/Catch Syntax

### Catch uses pattern matching, not variable binding
```nostos
# Correct - pattern matching syntax
try {
    riskyOperation()
} catch {
    "specific error" -> handleSpecific(),
    e -> handleGeneric(e)  # Catch-all pattern
}

# Wrong - this is NOT Nostos syntax
try {
    riskyOperation()
} catch (e) {
    handleError(e)
}
```

### Catch arms need commas
```nostos
# Correct
try { x } catch {
    "error1" -> handle1(),
    "error2" -> handle2(),
    _ -> handleOther()
}

# Wrong - missing commas
try { x } catch {
    "error1" -> handle1()
    "error2" -> handle2()
}
```

## Type System

### Generic syntax uses `[]`, not `<>`
```nostos
# Correct
List[Int]
Map[String, Int]
Option[T]

# Wrong - angle brackets are comparison operators
List<Int>  # Parsed as: List < Int > (comparison!)
```

### Type field access uses `.ty`, not `.type`
```nostos
# In templates, accessing field types:
template example(typeDef) = quote {
    ~typeDef.fields.map(f =>
        # Correct - use .ty
        eval(f.name ++ ": " ++ f.ty)
    )
}

# Wrong - type is a keyword
f.type  # Error!
```

### Records need type definitions
```nostos
# Correct - define type first
type Person = { name: String, age: Int }
p = Person("Alice", 30)

# Wrong - anonymous records don't exist
p = { name: "Alice", age: 30 }  # Error!
```

## Functions

### Single-expression functions don't need braces
```nostos
# Both are correct:
add(a, b) = a + b
add(a, b) = { a + b }

# But multi-statement needs braces
process(x) = {
    y = x * 2
    y + 1
}
```

### Return is implicit (last expression)
```nostos
# Correct - last expression is returned
calculate(x) = {
    y = x * 2
    z = y + 1
    z  # This is returned
}

# Explicit return exists but rarely needed
earlyExit(x) = {
    if x < 0 { return 0 } else { () }
    x * 2
}
```

### Pattern matching in function definitions
```nostos
# Multiple clauses with patterns
factorial(0) = 1
factorial(n) = n * factorial(n - 1)

# List patterns
sum([]) = 0
sum([h | t]) = h + sum(t)

# The clauses are tried in order
```

## Lists and Collections

### List cons pattern is `[h | t]`, not `h::t`
```nostos
# Correct
match list {
    [] -> "empty",
    [h | t] -> "head: " ++ show(h)
}

# Wrong - not Nostos syntax
match list {
    h::t -> "head: " ++ show(h)
}
```

### Map literals use `%{}`
```nostos
# Correct
myMap = %{"a": 1, "b": 2}

# Wrong - this is a block, not a map
myMap = {"a": 1, "b": 2}  # Error!
```

### Indexing with `[]` vs `.get()`
```nostos
# Direct indexing (may panic)
list[0]       # First element, panics if empty
map["key"]    # Value for key, panics if missing

# Safe access with Option
list.get(0)   # Some(first) or None
map.get("key") # Some(value) or None
```

## Common Runtime Errors

### Forgetting to handle None/Err
```nostos
# This will panic if list is empty
first = list[0]

# Safe alternative
first = match list.get(0) {
    Some(x) -> x,
    None -> defaultValue
}
```

### Integer division truncates
```nostos
5 / 2   # Returns 2, not 2.5

# For float division, use floats
5.0 / 2.0  # Returns 2.5
```

### String comparison is case-sensitive
```nostos
"Hello" == "hello"  # false

# For case-insensitive comparison
"Hello".toLower() == "hello".toLower()  # true
```

## Templates

### `~` splices AST, not values
```nostos
template example(fn) = quote {
    # ~fn.body inserts the AST of the function body
    result = ~fn.body
    result * 2
}

# The splice happens at compile time, not runtime
```

### eval() parses strings as code
```nostos
template makeFn(name) = quote {
    # eval turns a string into code
    ~eval(~name ++ "() = 42")
}

# This generates: myFunc() = 42
@makeFn("myFunc")
type Dummy = Dummy {}
```

### gensym for unique names
```nostos
template safe(fn) = quote {
    # Without gensym, variable names might collide
    ~gensym("tmp") = ~fn.body
    ~gensym("tmp")  # Different name each time!
}
```
