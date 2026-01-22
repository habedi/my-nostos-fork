# Traits in Nostos

## Defining Traits

```nostos
# Basic trait definition
trait Show
    show(self) -> String
end

trait Eq
    eq(self, other: Self) -> Bool
end

# Trait with multiple methods
trait Comparable
    compare(self, other: Self) -> Int
    lt(self, other: Self) -> Bool
    gt(self, other: Self) -> Bool
end
```

## Implementing Traits

```nostos
type Person = { name: String, age: Int }

# Implement Show for Person
Person: Show
    show(self) = self.name ++ " (age " ++ show(self.age) ++ ")"
end

# Implement Eq for Person
Person: Eq
    eq(self, other) = self.name == other.name && self.age == other.age
end

# Use the trait methods
alice = Person("Alice", 30)
bob = Person("Bob", 25)

alice.show()        # "Alice (age 30)"
alice.eq(bob)       # false
```

## Supertraits (Trait Inheritance)

```nostos
# Base trait
trait Displayable
    display(self) -> String
end

# Child trait requires Displayable
trait Formattable: Displayable
    format(self, prefix: String) -> String
end

type Item = { id: Int, name: String }

# Must implement Displayable FIRST
Item: Displayable
    display(self) = self.name
end

# Then can implement Formattable
Item: Formattable
    format(self, prefix) = prefix ++ ": " ++ self.display()
end
```

## Multiple Supertraits

```nostos
trait Printable
    print(self) -> String
end

trait Sortable
    sortKey(self) -> Int
end

# Requires both
trait Listable: Printable, Sortable
    listItem(self) -> String
end
```

## Trait Bounds on Functions

```nostos
# Function accepting any Show type
printItem[T: Show](item: T) = println(item.show())

# Multiple bounds with +
printAndCompare[T: Show + Eq](a: T, b: T) = {
    println(a.show())
    println(b.show())
    a.eq(b)
}

# Alternative syntax with when
process(item: T) when T: Show = item.show()
```

## Generic Trait Implementations

```nostos
type Box[T] = { value: T }

# Implement Show for Box when T has Show
Box[T]: Show when T: Show
    show(self) = "Box(" ++ self.value.show() ++ ")"
end

# Now works for any Box[T] where T: Show
intBox = Box(42)
intBox.show()       # "Box(42)"
```

## Operator Overloading via Traits

```nostos
# Num trait for arithmetic operators
trait Num
    add(self, other: Self) -> Self
    sub(self, other: Self) -> Self
    mul(self, other: Self) -> Self
    div(self, other: Self) -> Self
end

type Vec2 = { x: Int, y: Int }

Vec2: Num
    add(self, other) = Vec2(self.x + other.x, self.y + other.y)
    sub(self, other) = Vec2(self.x - other.x, self.y - other.y)
    mul(self, other) = Vec2(self.x * other.x, self.y * other.y)
    div(self, other) = Vec2(self.x / other.x, self.y / other.y)
end

# Now operators work!
v1 = Vec2(1, 2)
v2 = Vec2(3, 4)
v3 = v1 + v2        # Vec2(4, 6)
```

## Index Trait

```nostos
trait Index
    index(self, i: Int) -> Float
end

trait IndexMut
    indexMut(self, i: Int, value: Float) -> Self
end

type Vector = { data: List[Float] }

Vector: Index
    index(self, i) = self.data[i]
end

Vector: IndexMut
    indexMut(self, i, value) = Vector(self.data.set(i, value))
end

# Now bracket notation works
v = Vector([1.0, 2.0, 3.0])
v[0]            # 1.0
v[1] = 5.0      # Vector([1.0, 5.0, 3.0])
```

## Heterogeneous Collections with Sum Types

```nostos
trait Drawable
    draw(self) -> String
end

type Circle = { radius: Float }
type Square = { side: Float }

Circle: Drawable
    draw(self) = "Circle(r=" ++ show(self.radius) ++ ")"
end

Square: Drawable
    draw(self) = "Square(s=" ++ show(self.side) ++ ")"
end

# Sum type wrapper
type Shape = C(Circle) | S(Square)

Shape: Drawable
    draw(self) = match self { C(c) -> c.draw(), S(s) -> s.draw() }
end

# Now heterogeneous list works!
shapes: List[Shape] = [C(Circle(1.0)), S(Square(2.0))]
shapes.map(s => s.draw())   # ["Circle(r=1)", "Square(s=2)"]
```

## Built-in Traits

```nostos
# Show - convert to string
trait Show
    show(self) -> String
end

# Eq - equality comparison
trait Eq
    eq(self, other: Self) -> Bool
end

# Ord - ordering comparison
trait Ord
    lt(self, other: Self) -> Bool
    gt(self, other: Self) -> Bool
    lte(self, other: Self) -> Bool
    gte(self, other: Self) -> Bool
end

# Hash - for use in Maps/Sets
trait Hash
    hash(self) -> Int
end

# Copy - value can be copied (default for simple types)
trait Copy end

# Default - has a default value
trait Default
    default() -> Self
end
```

## Deriving Traits

```nostos
# Automatically derive common traits
type Point = { x: Int, y: Int }
    deriving (Show, Eq, Hash, Copy)

# Now these work automatically
p = Point(1, 2)
p.show()            # "Point { x: 1, y: 2 }"
p.eq(Point(1, 2))   # true
p.hash()            # some hash value
```
