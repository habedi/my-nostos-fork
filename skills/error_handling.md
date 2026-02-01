# Error Handling in Nostos

## Option Type

```nostos
type Option[T] = Some(T) | None

# Representing missing values
findUser(id: Int) -> Option[User] = {
    if id == 1 then Some(User("Alice"))
    else None
}

# Pattern matching
result = match findUser(1) {
    Some(user) -> "Found: " ++ user.name,
    None -> "Not found"
}
```

## Option Methods

```nostos
opt = Some(42)
none: Option[Int] = None

# Map - transform the value if present
opt.map(x => x * 2)         # Some(84)
none.map(x => x * 2)        # None

# getOrElse - extract with default
opt.getOrElse(0)            # 42
none.getOrElse(0)           # 0

# isSome / isNone
opt.isSome()                # true
none.isNone()               # true

# flatMap - for chained optionals
opt.flatMap(x => if x > 0 then Some(x) else None)
```

## Result Type

```nostos
type Result[T, E] = Ok(T) | Err(E)

# Representing success or failure
parseNumber(s: String) -> Result[Int, String] = {
    if s.all(c => c.isDigit()) then
        Ok(s.parseInt())
    else
        Err("Invalid number: " ++ s)
}

# Pattern matching
result = match parseNumber("42") {
    Ok(n) -> "Parsed: " ++ show(n),
    Err(e) -> "Error: " ++ e
}
```

## Result Methods

```nostos
ok: Result[Int, String] = Ok(42)
err: Result[Int, String] = Err("failed")

# Map - transform success value
ok.map(x => x * 2)          # Ok(84)
err.map(x => x * 2)         # Err("failed")

# mapErr - transform error value
ok.mapErr(e => "Error: " ++ e)   # Ok(42)
err.mapErr(e => "Error: " ++ e)  # Err("Error: failed")

# getOrElse
ok.getOrElse(0)             # 42
err.getOrElse(0)            # 0

# isOk / isErr
ok.isOk()                   # true
err.isErr()                 # true
```

## Chaining Results

```nostos
# Sequential operations that may fail
processData(input: String) -> Result[Int, String] = {
    match parseNumber(input) {
        Ok(n) -> {
            if n > 0 then Ok(n * 2)
            else Err("Number must be positive")
        },
        Err(e) -> Err(e)
    }
}

# Using flatMap for cleaner chaining
process(input: String) -> Result[Int, String] =
    parseNumber(input)
        .flatMap(n => if n > 0 then Ok(n) else Err("Must be positive"))
        .map(n => n * 2)
```

## Try/Catch for Exceptions

```nostos
# Some operations can throw exceptions
# Use try/catch to handle them

result = try {
    riskyOperation()
} catch e {
    "Error occurred: " ++ e
}

# Catch specific patterns
result = try {
    parseAndProcess(input)
} catch {
    "parse error" -> "Invalid input format",
    "not found" -> "Resource not found",
    e -> "Unknown error: " ++ e
}
```

## Throwing Exceptions

```nostos
# Throw an exception
divide(a: Int, b: Int) -> Int = {
    if b == 0 then
        throw "Division by zero"
    else
        a / b
}

# Will be caught by try/catch
result = try {
    divide(10, 0)
} catch e {
    0  # Default value on error
}
```

## Converting Between Types

```nostos
# Option to Result
optToResult(opt: Option[T], err: E) -> Result[T, E] = match opt {
    Some(x) -> Ok(x),
    None -> Err(err)
}

# Result to Option (loses error info)
resultToOpt(res: Result[T, E]) -> Option[T] = match res {
    Ok(x) -> Some(x),
    Err(_) -> None
}

# Example
findUser(id).optToResult("User not found")
```

## Early Return with Pattern Matching

```nostos
# Extract or return early
processUser(id: Int) -> Result[String, String] = {
    user = match findUser(id) {
        Some(u) -> u,
        None -> return Err("User not found")
    }

    profile = match getProfile(user) {
        Some(p) -> p,
        None -> return Err("Profile not found")
    }

    Ok(profile.summary)
}
```

## Collecting Results

```nostos
# Process list, collect all errors or all successes
processAll(items: List[String]) -> Result[List[Int], List[String]] = {
    results = items.map(parseNumber)
    errors = results.filter(r => r.isErr()).map(r => match r { Err(e) -> e, _ -> "" })

    if errors.isEmpty() then
        Ok(results.map(r => match r { Ok(x) -> x, _ -> 0 }))
    else
        Err(errors)
}
```

## Best Practices

```nostos
# 1. Prefer Result over exceptions for expected errors
parseConfig(path: String) -> Result[Config, String]  # Good
# vs throwing exceptions                              # Avoid

# 2. Use Option for "might not exist"
findById(id: Int) -> Option[User]

# 3. Use Result for "might fail with reason"
saveUser(user: User) -> Result[(), String]

# 4. Provide context in errors
Err("Failed to parse config at line " ++ show(line) ++ ": " ++ reason)

# 5. Use early return for cleaner error handling
processRequest(req: Request) -> Result[Response, Error] = {
    user = match authenticate(req) {
        Ok(u) -> u,
        Err(e) -> return Err(AuthError(e))
    }

    data = match fetchData(user) {
        Ok(d) -> d,
        Err(e) -> return Err(DataError(e))
    }

    Ok(Response(data))
}
```

## Custom Error Types

```nostos
# Define specific error variants
type AppError =
    | NotFound(String)
    | InvalidInput(String)
    | NetworkError(String)
    | DatabaseError(String)

# Use in Result
fetchUser(id: Int) -> Result[User, AppError] = {
    if id < 0 then
        Err(InvalidInput("ID must be positive"))
    else if id > 1000 then
        Err(NotFound("User " ++ show(id)))
    else
        Ok(User("User" ++ show(id)))
}

# Handle specific errors
match fetchUser(id) {
    Ok(user) -> handleUser(user),
    Err(NotFound(msg)) -> show404(msg),
    Err(InvalidInput(msg)) -> show400(msg),
    Err(e) -> show500(show(e))
}
```

## See Also

- **types.md** - Defining custom error types with variants
- **concurrency.md** - Error handling in spawned processes, process linking for crash handling
- **templates.md** - `@withFallback` and `@retry` patterns for automatic error recovery
- **02_stdlib_reference.md** - Full Option and Result method reference
