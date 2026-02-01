# Templates & Metaprogramming in Nostos

Templates let you write code that generates code at compile time. This is powerful for eliminating boilerplate, creating DSLs, and building reusable patterns.

## Core Concepts

**Templates** are compile-time functions that manipulate code as data:
- `quote { code }` - captures code as AST (not executed, just stored)
- `~expr` - splices an AST value into a quote (inserts the code)
- `@decorator` - applies a template to a function or type

## Function Decorators

Decorators wrap or transform functions at compile time:

```nostos
# Double the return value of any function
template double(fn) = quote {
    result = ~fn.body    # splice in the original function body
    result * 2
}

@double
getValue() = 21

main() = getValue()  # Returns 42 (21 * 2)
```

**Available function metadata:**
- `~fn.name` - function name as String
- `~fn.body` - the function body (AST)
- `~fn.params` - list of parameters with name/type
- `~fn.returnType` - return type as String

## Type Decorators

Generate code based on type structure:

```nostos
# Auto-generate getters for all fields
template withGetters(typeDef) = quote {
    ~typeDef
    ~typeDef.fields.map(f =>
        eval("get_" ++ f.name ++ "(r: " ++ ~typeDef.name ++ ") = r." ++ f.name))
}

@withGetters
type Point = Point { x: Int, y: Int }

main() = {
    p = Point(x: 10, y: 20)
    get_x(p) + get_y(p)  # 30
}
```

**Type introspection:**
- `~typeDef.name` - type name as String
- `~typeDef.fields` - list of fields (for single-constructor types)
- `~typeDef.fields[i].name` - field name at index
- `~typeDef.fields[i].ty` - field type at index (note: `ty` not `type`)

## Code Generation with eval

`eval("code")` parses a string as code at compile time:

```nostos
# Generate a function dynamically
template makeAdder(typeDef, amount) = quote {
    ~typeDef
    ~eval("add" ++ ~amount ++ "(x: Int) = x + " ++ ~amount)
}

@makeAdder("10")
type Config = Config {}

main() = add10(5)  # 15
```

## Compile-Time Conditionals

Use `~if` for conditional code generation:

```nostos
template maybeLog(fn, shouldLog) = quote {
    ~if ~shouldLog {
        quote {
            println("Calling " ++ ~fn.name)
            ~fn.body
        }
    } else {
        quote { ~fn.body }
    }
}

@maybeLog(true)
debug() = 42      # prints "Calling debug", returns 42

@maybeLog(false)
release() = 42    # just returns 42, no logging
```

## Exception Handling in Templates

Add try/catch for error handling patterns:

```nostos
# Wrap any function with a fallback value
template withFallback(fn, fallback) = quote {
    try {
        ~fn.body
    } catch {
        _ -> ~fallback
    }
}

@withFallback("error")
risky() = throw("oops")

main() = risky()  # Returns "error" instead of throwing
```

## Retry Pattern

Retry failed operations multiple times:

```nostos
template retry3(fn) = quote {
    try { ~fn.body } catch {
        _ -> try { ~fn.body } catch {
            _ -> ~fn.body
        }
    }
}

mvar attempts: Int = 0

@retry3
flaky() = {
    attempts = attempts + 1
    if attempts < 3 { throw("not ready") }
    "success"
}

main() = flaky()  # Succeeds on 3rd attempt
```

## Unique Identifiers with gensym

Avoid naming collisions in generated code:

```nostos
template withHelper(typeDef) = quote {
    ~typeDef
    ~eval(~gensym("helper") ++ "() = 42")
}

@withHelper
type A = A {}

@withHelper
type B = B {}

# Generates helper_0() and helper_1() - no collision!
main() = helper_0() + helper_1()  # 84
```

## Parameter Access with param()

Reference function parameters with `~param(n)`:

```nostos
template validatePositive(fn) = quote {
    if ~param(0) <= 0 {
        throw(~fn.name ++ ": must be positive")
    }
    ~fn.body
}

@validatePositive
sqrt(n: Int) = n * n

main() = sqrt(5)    # 25
# sqrt(-1) would throw "sqrt: must be positive"
```

The `~param(n)` shorthand is equivalent to `~toVar(fn.params[n].name)`.

## Compile-Time Computation

Execute code at compile time with `comptime`:

```nostos
# String syntax
template withDefault(fn, expr) = quote {
    value = ~comptime(~expr)  # evaluated at compile time
    ~fn.body + value
}

@withDefault("21 * 2")
add(x: Int) = x

main() = add(0)  # 42

# Block syntax
template computed(fn, useSquare) = quote {
    ~comptime({
        base = 10
        if ~useSquare { base * base } else { base * 2 }
    })
}
```

## Practical Patterns

### Logging Decorator
```nostos
template logged(fn) = quote {
    println(">>> " ++ ~fn.name)
    result = ~fn.body
    println("<<< " ++ ~fn.name)
    result
}
```

### Builder Pattern
```nostos
template builder(typeDef) = quote {
    ~typeDef
    ~typeDef.fields.map(f =>
        eval("with_" ++ f.name ++ "(r: " ++ ~typeDef.name ++ ", v: " ++ f.ty ++ ") = " ++
             ~typeDef.name ++ "(" ++ f.name ++ ": v)"))
}

@builder
type Config = Config { timeout: Int, debug: Bool }

# Generates: with_timeout(r, v), with_debug(r, v)
```

### Feature Flags
```nostos
template featureFlag(fn, enabled, msg) = quote {
    ~if ~enabled {
        quote { ~fn.body }
    } else {
        quote { throw(~msg) }
    }
}

@featureFlag(true, "Beta disabled")
betaFeature() = "works!"

@featureFlag(false, "Experimental disabled")
experimental() = "never runs"
```

## Key Points for Code Generation

1. **Templates run at compile time** - no runtime overhead
2. **`quote` captures code** - doesn't execute it
3. **`~` splices values** - inserts AST into quotes
4. **`eval` parses strings** - for dynamic function names
5. **`gensym` creates unique names** - prevents collisions
6. **`toVar` references parameters** - for validation patterns
7. **`~if` generates conditionally** - compile-time branching
8. **`comptime` executes early** - pre-compute values
9. **try/catch works in templates** - for error handling patterns

## See Also

- **types.md** - Type definitions that templates can introspect (`~typeDef.fields`)
- **traits.md** - Templates can generate trait implementations
- **functions.md** - Function syntax that templates transform (`~fn.body`, `~fn.params`)
- **error_handling.md** - Try/catch syntax used in template error patterns
