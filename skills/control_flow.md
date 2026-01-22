# Control Flow in Nostos

## If/Then/Else

```nostos
# Expression form (returns a value)
max = if a > b then a else b

# Statement form with blocks
result = if condition then {
    doSomething()
    value1
} else {
    doOther()
    value2
}

# Nested conditionals
grade = if score >= 90 then "A"
        else if score >= 80 then "B"
        else if score >= 70 then "C"
        else "F"

# If without else (returns unit)
if debug then
    println("Debug mode")
else
    ()
```

## Pattern Matching (match)

```nostos
# Basic match
describe(n: Int) = match n {
    0 -> "zero",
    1 -> "one",
    _ -> "other"
}

# Match on variants
type Option[T] = Some(T) | None

getValue(opt: Option[Int]) = match opt {
    Some(x) -> x,
    None -> 0
}

# Match with guards
classify(n: Int) = match n {
    x if x < 0 -> "negative",
    0 -> "zero",
    x if x > 100 -> "large",
    _ -> "positive"
}

# Match on tuples
handlePoint((x, y)) = match (x, y) {
    (0, 0) -> "origin",
    (0, _) -> "on y-axis",
    (_, 0) -> "on x-axis",
    _ -> "elsewhere"
}

# Match on lists
describe(lst) = match lst {
    [] -> "empty",
    [x] -> "single: " ++ show(x),
    [x, y] -> "pair",
    [h | t] -> "head: " ++ show(h) ++ ", tail has " ++ show(t.length())
}

# Match on records
type Person = { name: String, age: Int }

greet(p: Person) = match p {
    { name: "Alice", age } -> "Hi Alice, you're " ++ show(age),
    { name, age } if age < 18 -> "Hello young " ++ name,
    { name, _ } -> "Hello " ++ name
}
```

## While Loops

```nostos
# Basic while loop
main() = {
    var i = 0
    while i < 5 {
        println(show(i))
        i = i + 1
    }
}

# Sum with while
sumTo(n: Int) -> Int = {
    var sum = 0
    var i = 1
    while i <= n {
        sum = sum + i
        i = i + 1
    }
    sum
}

# Early exit with return
findIndex(items: List[Int], target: Int) -> Int = {
    var i = 0
    while i < items.length() {
        if items[i] == target then
            return i
        else
            ()
        i = i + 1
    }
    -1
}
```

## Functional Iteration (Preferred)

```nostos
# Instead of while loops, prefer functional style:

# Map over a list
doubled = [1, 2, 3].map(x => x * 2)

# Filter elements
evens = [1, 2, 3, 4].filter(x => x % 2 == 0)

# Reduce/fold
sum = [1, 2, 3].fold(0, (acc, x) => acc + x)

# Range-based iteration
# Create a range and process
range(1, 5).map(x => x * x)     # [1, 4, 9, 16]
range(1, 10).filter(x => x % 2 == 0)  # [2, 4, 6, 8]

# forEach for side effects
[1, 2, 3].forEach(x => println(show(x)))
```

## Early Return

```nostos
# Return exits the function immediately
checkAge(age: Int) -> String = {
    if age < 0 then
        return "Invalid age"
    else
        ()

    if age < 18 then
        return "Minor"
    else
        ()

    "Adult"
}

# Works in nested contexts
process(items: List[Int]) -> Int = {
    var total = 0
    var i = 0
    while i < items.length() {
        item = items[i]
        if item < 0 then
            return -1  # Error: negative found
        else
            ()
        total = total + item
        i = i + 1
    }
    total
}
```

## Blocks

```nostos
# Blocks are expressions, return last value
result = {
    x = 10
    y = 20
    x + y   # This is returned
}

# Nested blocks
outer = {
    a = {
        temp = 5
        temp * 2
    }
    b = {
        temp = 3
        temp * 3
    }
    a + b   # 10 + 9 = 19
}
```

## Match as Expression

```nostos
# Match returns a value
status = match code {
    200 -> "OK",
    404 -> "Not Found",
    500 -> "Server Error",
    _ -> "Unknown"
}

# Used inline
println("Status: " ++ match code { 200 -> "OK", _ -> "Error" })
```

## Combining Control Flow

```nostos
processItems(items: List[Int]) -> String = {
    if items.length() == 0 then
        return "Empty list"
    else
        ()

    var result = ""
    var i = 0
    while i < items.length() {
        item = items[i]
        category = match item {
            x if x < 0 -> "negative",
            0 -> "zero",
            x if x > 100 -> "large",
            _ -> "normal"
        }
        result = result ++ category ++ " "
        i = i + 1
    }
    result
}
```
