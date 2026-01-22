# Lists in Nostos

## Creating Lists

```nostos
# Empty list
empty = []

# List with elements
numbers = [1, 2, 3, 4, 5]
strings = ["a", "b", "c"]
mixed = [1, "two", true]   # Heterogeneous (not recommended)

# Type annotation
nums: List[Int] = [1, 2, 3]
```

## Cons Operator (Prepend)

```nostos
# | prepends element to list
list = [1 | [2, 3]]         # [1, 2, 3]
list = [0 | [1, 2, 3]]      # [0, 1, 2, 3]

# Building lists
addFront(x, lst) = [x | lst]
addFront(0, [1, 2])         # [0, 1, 2]
```

## Pattern Matching on Lists

```nostos
# Match empty vs non-empty
describe([]) = "empty"
describe([h | t]) = "head: " ++ show(h)

# Match specific lengths
handleList([]) = "empty"
handleList([x]) = "single: " ++ show(x)
handleList([x, y]) = "pair: " ++ show(x) ++ ", " ++ show(y)
handleList(_) = "many elements"

# Extract head and tail
firstTwo([a, b | _]) = (a, b)
```

## Basic Operations

```nostos
lst = [1, 2, 3, 4, 5]

# Length
lst.length()            # 5

# Access by index (0-based)
lst[0]                  # 1
lst[2]                  # 3
lst.get(0)              # 1

# Head and tail
lst.head()              # Some(1)
lst.tail()              # [2, 3, 4, 5]
lst.last()              # Some(5)

# Check empty
lst.isEmpty()           # false
[].isEmpty()            # true
```

## Transformations

```nostos
# Map: transform each element
[1, 2, 3].map(x => x * 2)           # [2, 4, 6]
["a", "b"].map(s => s.toUpper())    # ["A", "B"]

# Filter: keep matching elements
[1, 2, 3, 4].filter(x => x % 2 == 0)    # [2, 4]
["apple", "banana", "apricot"].filter(s => s.startsWith("a"))  # ["apple", "apricot"]

# Fold/Reduce: combine into single value
[1, 2, 3, 4].fold(0, (acc, x) => acc + x)   # 10 (sum)
[1, 2, 3, 4].fold(1, (acc, x) => acc * x)   # 24 (product)
```

## More List Methods

```nostos
# Take/Drop
[1, 2, 3, 4, 5].take(3)     # [1, 2, 3]
[1, 2, 3, 4, 5].drop(2)     # [3, 4, 5]

# Reverse
[1, 2, 3].reverse()         # [3, 2, 1]

# Concatenation
[1, 2] ++ [3, 4]            # [1, 2, 3, 4]
[[1, 2], [3, 4]].flatten()  # [1, 2, 3, 4]

# Sort
[3, 1, 4, 1, 5].sort()      # [1, 1, 3, 4, 5]

# Unique
[1, 2, 2, 3, 3, 3].unique() # [1, 2, 3]
```

## Searching

```nostos
lst = [10, 20, 30, 40]

# Find first match
lst.find(x => x > 15)       # Some(20)
lst.find(x => x > 100)      # None

# Check existence
lst.any(x => x > 30)        # true
lst.all(x => x > 5)         # true
lst.contains(20)            # true

# Index of
lst.indexOf(30)             # 2
lst.indexOf(99)             # -1
```

## FlatMap

```nostos
# Map then flatten
[[1, 2], [3, 4]].flatMap(lst => lst.map(x => x * 2))
# [2, 4, 6, 8]

# Useful for optional results
users.flatMap(u => u.email)  # Skips None values
```

## Zipping

```nostos
# Combine two lists element-wise
zip([1, 2, 3], ["a", "b", "c"])  # [(1, "a"), (2, "b"), (3, "c")]

# Zip with function
zipWith((a, b) => a + b, [1, 2], [10, 20])  # [11, 22]
```

## Partitioning

```nostos
# Split by predicate
(evens, odds) = [1, 2, 3, 4, 5].partition(x => x % 2 == 0)
# evens = [2, 4], odds = [1, 3, 5]

# Group by
words = ["apple", "banana", "apricot", "blueberry"]
words.groupBy(w => w.charAt(0))
# {'a': ["apple", "apricot"], 'b': ["banana", "blueberry"]}
```

## Recursive List Functions

```nostos
# Sum
sum([]) = 0
sum([h | t]) = h + sum(t)

# Length
len([]) = 0
len([_ | t]) = 1 + len(t)

# Map (manual implementation)
myMap([], _) = []
myMap([h | t], f) = [f(h) | myMap(t, f)]

# Filter (manual)
myFilter([], _) = []
myFilter([h | t], p) =
    if p(h) then [h | myFilter(t, p)]
    else myFilter(t, p)
```

## Method Chaining

```nostos
# Fluent API style
result = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    .filter(x => x % 2 == 0)      # [2, 4, 6, 8, 10]
    .map(x => x * x)              # [4, 16, 36, 64, 100]
    .take(3)                      # [4, 16, 36]
    .fold(0, (a, b) => a + b)     # 56

# Processing data
orders
    .filter(o => o.status == "completed")
    .map(o => o.total)
    .fold(0.0, (a, b) => a + b)
```

## Ranges

```nostos
# Create a range
range(1, 5)         # [1, 2, 3, 4]
range(0, 10)        # [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# With step
rangeStep(0, 10, 2) # [0, 2, 4, 6, 8]

# Common patterns
range(1, n + 1).map(x => x * x)  # Squares from 1 to n
range(0, n).fold(1, (acc, _) => acc * base)  # Power
```

## ForEach (Side Effects)

```nostos
# When you need side effects
[1, 2, 3].forEach(x => println(show(x)))

# vs map which returns values
results = [1, 2, 3].map(x => {
    println("Processing: " ++ show(x))
    x * 2
})
```
