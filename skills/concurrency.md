# Concurrency in Nostos

## Spawning Processes

```nostos
# spawn creates a lightweight process
pid = spawn(() => {
    println("Hello from spawned process!")
})

# Process runs concurrently with main
println("Main continues immediately")

# Spawn with argument
worker(id: Int) = {
    println("Worker " ++ show(id) ++ " started")
}

spawn(() => worker(1))
spawn(() => worker(2))
```

## Message Passing

```nostos
# Processes communicate via messages
# self() returns current process ID

# Send a message
send(pid, "Hello!")

# Receive a message (blocks until received)
msg = receive()

# Basic ping-pong
main() = {
    parent = self()

    child = spawn(() => {
        msg = receive()
        send(parent, "Got: " ++ msg)
    })

    send(child, "Hello")
    reply = receive()
    println(reply)      # "Got: Hello"
}
```

## Receive with Pattern Matching

```nostos
# Receive with pattern matching
result = receive {
    "ping" -> "pong",
    ("add", a, b) -> show(a + b),
    ("quit") -> {
        println("Shutting down")
        "bye"
    },
    other -> "Unknown: " ++ show(other)
}
```

## Receive with Timeout

```nostos
# Receive with timeout (milliseconds)
result = receiveTimeout(1000) {
    msg -> "Got: " ++ msg
}

# Returns Option - None if timeout
match result {
    Some(value) -> println(value),
    None -> println("Timed out!")
}
```

## MVar (Mutable Variable)

```nostos
# MVar is a synchronized mutable container
# Can be empty or full

# Create empty MVar
mv = MVar.new()

# Create MVar with initial value
mv = MVar.newWith(42)

# Put value (blocks if full)
mv.put(100)

# Take value (blocks if empty)
value = mv.take()

# Read without removing
value = mv.read()

# Try operations (non-blocking, returns Option)
result = mv.tryTake()
result = mv.tryPut(42)
```

## Worker Pool Pattern

```nostos
# Process pool for parallel work

workerLoop(id: Int) = {
    match receive() {
        ("task", data, replyTo) -> {
            result = processData(data)
            send(replyTo, ("result", id, result))
            workerLoop(id)
        },
        "stop" -> ()
    }
}

createWorkers(n: Int) -> List[Pid] = {
    range(1, n + 1).map(id => spawn(() => workerLoop(id)))
}

main() = {
    workers = createWorkers(4)
    me = self()

    # Distribute work
    tasks = [1, 2, 3, 4, 5, 6, 7, 8]
    tasks.zip(cycle(workers)).forEach((task, worker) => {
        send(worker, ("task", task, me))
    })

    # Collect results
    results = tasks.map(_ => receive())
    println(show(results))

    # Stop workers
    workers.forEach(w => send(w, "stop"))
}
```

## Parallel Map

```nostos
# Parallel map over a list
parallelMap(items: List[T], f: T -> R) -> List[R] = {
    me = self()

    # Spawn a process for each item
    pids = items.map(item => spawn(() => {
        result = f(item)
        send(me, result)
    }))

    # Collect results (in order)
    items.map(_ => receive())
}

# Usage
squares = parallelMap([1, 2, 3, 4, 5], x => x * x)
```

## Process Linking

```nostos
# Link processes - if one dies, the other is notified
spawnLink(() => {
    # If this crashes, parent receives exit message
    riskyOperation()
})

# Handle exit messages
result = receive {
    ("EXIT", pid, reason) -> "Process " ++ show(pid) ++ " exited: " ++ reason,
    normalMsg -> handleNormal(normalMsg)
}
```

## Supervisors

```nostos
# Restart failed processes
supervisor(childFn) = {
    child = spawnLink(childFn)

    match receive() {
        ("EXIT", _, _) -> {
            println("Child crashed, restarting...")
            supervisor(childFn)
        },
        msg -> {
            send(child, msg)
            supervisor(childFn)
        }
    }
}
```

## Ring Benchmark

```nostos
# Classic concurrency benchmark: message ring

ringNode(next: Pid) = {
    match receive() {
        0 -> send(next, 0),
        n -> {
            send(next, n - 1)
            ringNode(next)
        }
    }
}

main() = {
    n = 1000    # Ring size
    m = 10000   # Messages

    # Create ring of processes
    first = self()
    last = range(1, n).fold(first, (prev, _) => {
        spawn(() => ringNode(prev))
    })

    # Connect last to first
    send(last, m)

    # Wait for completion
    receive()
    println("Ring complete!")
}
```

## Async Sleep

```nostos
# Sleep without blocking other processes
sleep(1000)     # Sleep 1 second

# Delayed message
sendAfter(pid, msg, delay) = {
    spawn(() => {
        sleep(delay)
        send(pid, msg)
    })
}

# Timer pattern
startTimer(duration: Int, callback: () -> ()) = {
    spawn(() => {
        sleep(duration)
        callback()
    })
}
```

## Channel Pattern

```nostos
# Implement channels using MVars
type Channel[T] = { queue: MVar[List[T]] }

newChannel() -> Channel[T] = Channel(MVar.newWith([]))

channelSend(ch: Channel[T], value: T) = {
    items = ch.queue.take()
    ch.queue.put(items ++ [value])
}

channelReceive(ch: Channel[T]) -> T = {
    items = ch.queue.take()
    match items {
        [h | t] -> {
            ch.queue.put(t)
            h
        },
        [] -> {
            ch.queue.put([])
            # Wait and retry
            sleep(10)
            channelReceive(ch)
        }
    }
}
```

## Best Practices

```nostos
# 1. Prefer message passing over shared state
# Good: send(worker, data)
# Avoid: shared mutable variables

# 2. Use timeouts to prevent deadlocks
receiveTimeout(5000) { ... }

# 3. Handle process failures
spawnLink for supervised processes

# 4. Keep messages small
# Send IDs, not large data structures

# 5. Use MVars for simple synchronization
counter = MVar.newWith(0)
increment() = {
    n = counter.take()
    counter.put(n + 1)
}
```

## See Also

- **error_handling.md** - Handle exceptions in spawned processes with try/catch
- **03_examples.md** - End-to-end concurrent task processing example
- **templates.md** - Use `@retry` template pattern for flaky concurrent operations
