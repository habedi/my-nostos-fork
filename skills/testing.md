# Testing in Nostos

## Basic Assertions

```nostos
# Assert condition is true
assert(1 + 1 == 2)

# Assert with message
assert(user.age >= 18, "User must be adult")

# Assert equality
assert_eq(actual, expected)
assert_eq(sum([1, 2, 3]), 6)

# Assert not equal
assert_ne(a, b)
```

## Test File Format

Test files use special comments to specify expected behavior:

```nostos
# expect: 42
# This test expects main() to return 42

main() = 21 * 2
```

```nostos
# expect_error: type mismatch
# This test expects a compilation error containing "type mismatch"

main() = "hello" + 5
```

## Organizing Tests

```nostos
# Group related assertions
testArithmetic() = {
    assert_eq(1 + 1, 2)
    assert_eq(10 - 3, 7)
    assert_eq(4 * 5, 20)
    assert_eq(15 / 3, 5)
    println("Arithmetic tests passed!")
}

testStrings() = {
    assert_eq("hello" ++ " world", "hello world")
    assert_eq("abc".length(), 3)
    assert_eq("HELLO".toLower(), "hello")
    println("String tests passed!")
}

main() = {
    testArithmetic()
    testStrings()
    0
}
```

## Testing with Setup/Teardown

```nostos
# Setup helper
withTestData(test: List[Int] -> T) -> T = {
    data = [1, 2, 3, 4, 5]  # Setup
    result = test(data)
    # Cleanup (if needed)
    result
}

# Usage
main() = {
    withTestData(data => {
        assert_eq(data.length(), 5)
        assert_eq(data.sum(), 15)
    })
    0
}
```

## Testing Option/Result

```nostos
# Test Option values
testOption() = {
    some = Some(42)
    none: Option[Int] = None

    assert(some.isSome())
    assert(none.isNone())
    assert_eq(some.getOrElse(0), 42)
    assert_eq(none.getOrElse(0), 0)
}

# Test Result values
testResult() = {
    ok: Result[Int, String] = Ok(42)
    err: Result[Int, String] = Err("failed")

    assert(ok.isOk())
    assert(err.isErr())
    assert_eq(ok.getOrElse(0), 42)
    assert_eq(err.getOrElse(0), 0)
}
```

## Testing Exceptions

```nostos
# Test that exception is thrown
testThrows() = {
    threw = try {
        divide(10, 0)
        false
    } catch _ {
        true
    }
    assert(threw, "Should have thrown")
}

# Test exception message
testErrorMessage() = {
    message = try {
        parseNumber("abc")
        ""
    } catch e {
        e
    }
    assert(message.contains("invalid"), "Should mention 'invalid'")
}
```

## Property-Based Testing Pattern

```nostos
# Test properties hold for many inputs
testProperty(name: String, gen: () -> T, prop: T -> Bool, iterations: Int) = {
    for i = 0 to iterations {
        value = gen()
        if !prop(value) then {
            println("Property '" ++ name ++ "' failed for: " ++ show(value))
            assert(false)
        }
    }
    println("Property '" ++ name ++ "' passed " ++ show(iterations) ++ " tests")
}

# Usage
main() = {
    # Test that reversing a list twice gives original
    testProperty(
        "reverse-reverse",
        () => randomList(10),
        list => list.reverse().reverse() == list,
        100
    )
    0
}
```

## Testing Async/Concurrent Code

```nostos
# Test with timeout
testWithTimeout(name: String, timeoutMs: Int, test: () -> T) = {
    me = self()
    spawn(() => {
        result = test()
        send(me, ("done", result))
    })

    match receiveTimeout(timeoutMs) {
        Some(("done", result)) -> {
            println("Test '" ++ name ++ "' passed")
            result
        }
        None -> {
            println("Test '" ++ name ++ "' timed out!")
            assert(false)
        }
    }
}

# Test message passing
testMessagePassing() = {
    parent = self()
    child = spawn(() => {
        msg = receive()
        send(parent, msg * 2)
    })
    send(child, 21)
    result = receive()
    assert_eq(result, 42)
}
```

## Mocking Pattern

```nostos
# Define interface as functions
type Database = {
    query: String -> List[Row],
    execute: String -> Int
}

# Real implementation
realDb = Database {
    query: sql => Pg.query(conn, sql, []),
    execute: sql => Pg.execute(conn, sql, [])
}

# Mock implementation
mockDb = Database {
    query: _ => [("Alice", 30), ("Bob", 25)],
    execute: _ => 1
}

# Function uses interface
getUsers(db: Database) = db.query("SELECT * FROM users")

# Test with mock
testGetUsers() = {
    users = getUsers(mockDb)
    assert_eq(users.length(), 2)
}
```

## Test Fixtures

```nostos
# Create test data
createTestUser(id: Int) = User {
    id: id,
    name: "User" ++ show(id),
    email: "user" ++ show(id) ++ "@test.com"
}

createTestUsers(n: Int) = range(1, n + 1).map(createTestUser)

# Usage in tests
testUserFiltering() = {
    users = createTestUsers(10)
    adults = users.filter(u => u.id > 5)
    assert_eq(adults.length(), 5)
}
```

## Testing HTTP Handlers

```nostos
# Create mock request
mockRequest(path: String, method: String, body: String) = {
    Request {
        path: path,
        method: method,
        body: body,
        headers: %{},
        query: %{},
        id: 0
    }
}

# Capture response
type CapturedResponse = { status: Int, body: String }
mvar capturedResponse: Option[CapturedResponse] = None

mockRespond(req, status: Int, body: String) = {
    capturedResponse = Some(CapturedResponse { status, body })
}

# Test handler
testGetUsersEndpoint() = {
    req = mockRequest("/users", "GET", "")
    handler(req)  # Uses mockRespond

    match capturedResponse {
        Some(resp) -> {
            assert_eq(resp.status, 200)
            assert(resp.body.contains("Alice"))
        }
        None -> assert(false, "No response captured")
    }
}
```

## Benchmark Pattern

```nostos
benchmark(name: String, iterations: Int, fn: () -> T) = {
    start = currentTimeMillis()
    for i = 0 to iterations {
        fn()
    }
    elapsed = currentTimeMillis() - start
    perOp = elapsed / iterations
    println(name ++ ": " ++ show(elapsed) ++ "ms total, " ++ show(perOp) ++ "ms/op")
}

# Usage
main() = {
    benchmark("list append", 10000, () => {
        [1, 2, 3] ++ [4, 5, 6]
    })

    benchmark("map insert", 10000, () => {
        %{"a": 1}.insert("b", 2)
    })
    0
}
```

## Test Runner Pattern

```nostos
type TestResult = Pass(String) | Fail(String, String)

runTest(name: String, test: () -> ()) -> TestResult = {
    try {
        test()
        Pass(name)
    } catch e {
        Fail(name, e)
    }
}

runSuite(tests: List[(String, () -> ())]) = {
    results = tests.map((name, test) => runTest(name, test))

    passed = results.filter(r => match r { Pass(_) -> true, _ -> false }).length()
    failed = results.filter(r => match r { Fail(_, _) -> true, _ -> false }).length()

    results.each(r => match r {
        Pass(name) -> println("[PASS] " ++ name)
        Fail(name, err) -> println("[FAIL] " ++ name ++ ": " ++ err)
    })

    println("\n" ++ show(passed) ++ " passed, " ++ show(failed) ++ " failed")
    failed == 0
}

# Usage
main() = {
    success = runSuite([
        ("arithmetic", testArithmetic),
        ("strings", testStrings),
        ("options", testOption)
    ])
    if success then 0 else 1
}
```
