# Reactive Programming in Nostos

## Reactive Records

Reactive records automatically track changes to their fields:

```nostos
# Define a reactive type
reactive Counter = { value: Int }

main() = {
    counter = Counter { value: 0 }

    # Register change listener
    counter.onChange("value", (old, new) => {
        println("Value changed: " ++ show(old) ++ " -> " ++ show(new))
    })

    # Changes trigger the callback
    counter.value = 1   # "Value changed: 0 -> 1"
    counter.value = 5   # "Value changed: 1 -> 5"
}
```

## Multiple Callbacks

```nostos
reactive State = { count: Int, name: String }

main() = {
    state = State { count: 0, name: "initial" }

    # Multiple callbacks on same field
    state.onChange("count", (_, new) => println("Logger: count = " ++ show(new)))
    state.onChange("count", (_, new) => updateUI(new))

    # Callbacks on different fields
    state.onChange("name", (_, new) => println("Name changed to: " ++ new))

    state.count = 42    # Both count callbacks fire
    state.name = "updated"  # Name callback fires
}
```

## Change History Tracking

```nostos
reactive Document = { title: String, content: String }

mvar history: List[(String, String, String)] = []

trackChanges(doc: Document) = {
    doc.onChange("title", (old, new) => {
        history = history ++ [("title", old, new)]
    })
    doc.onChange("content", (old, new) => {
        history = history ++ [("content", old, new)]
    })
}

main() = {
    doc = Document { title: "Untitled", content: "" }
    trackChanges(doc)

    doc.title = "My Document"
    doc.content = "Hello, world!"
    doc.title = "My Great Document"

    # Print change history
    history.each((field, old, new) => {
        println(field ++ ": '" ++ old ++ "' -> '" ++ new ++ "'")
    })
}
```

## Reactive Variants

```nostos
reactive Status = Idle | Loading | Success(String) | Error(String)

main() = {
    status = Idle

    # Track state transitions
    status.onChange((old, new) => {
        println("Status: " ++ show(old) ++ " -> " ++ show(new))
    })

    status.set(Loading)           # "Status: Idle -> Loading"
    status.set(Success("Done"))   # "Status: Loading -> Success(Done)"
}
```

## State Machine Pattern

```nostos
reactive ConnectionState =
    | Disconnected
    | Connecting
    | Connected { sessionId: String }
    | Reconnecting { attempt: Int }

handleConnection(state: ConnectionState) = {
    state.onChange((old, new) => {
        match new {
            Connected { sessionId } -> {
                println("Connected! Session: " ++ sessionId)
                startHeartbeat()
            }
            Disconnected -> {
                println("Disconnected")
                stopHeartbeat()
            }
            Reconnecting { attempt } -> {
                println("Reconnecting... attempt " ++ show(attempt))
            }
            _ -> ()
        }
    })
}
```

## RWeb: Reactive Web Framework

```nostos
use stdlib.rweb.*

# Define reactive state for the page
reactive AppState = { count: Int, message: String }

# Session setup function
sessionSetup(writerId) = {
    state = AppState { count: 0, message: "Welcome!" }

    # Render function - called on state changes
    renderPage = () => RHtml(div([
        h1(state.message),
        p("Count: " ++ show(state.count)),
        button("Increment", dataAction: "increment"),
        button("Reset", dataAction: "reset")
    ]))

    # Action handler
    onAction = (action, params) => match action {
        "increment" -> { state.count = state.count + 1 }
        "reset" -> { state.count = 0 }
        _ -> ()
    }

    (renderPage, onAction)
}

main() = startRWeb(8080, "Counter App", sessionSetup)
```

## Component Pattern

```nostos
use stdlib.rweb.*

# Reusable counter component
counterComponent(state, fieldName: String) = {
    count = state[fieldName]

    component("counter-" ++ fieldName, () => RHtml(div(class: "counter", [
        span(show(count)),
        button("+", dataAction: "inc-" ++ fieldName),
        button("-", dataAction: "dec-" ++ fieldName)
    ])))
}

# Use in page
renderPage = () => RHtml(div([
    h1("Multi-Counter"),
    counterComponent(state, "counter1"),
    counterComponent(state, "counter2"),
    counterComponent(state, "counter3")
]))
```

## Form Handling

```nostos
use stdlib.rweb.*

reactive FormState = {
    username: String,
    email: String,
    errors: Map[String, String]
}

validate(state: FormState) = {
    errors = %{}
    if state.username.length() < 3 then {
        errors["username"] = "Username too short"
    }
    if !state.email.contains("@") then {
        errors["email"] = "Invalid email"
    }
    state.errors = errors
    errors.isEmpty()
}

sessionSetup(writerId) = {
    state = FormState { username: "", email: "", errors: %{} }

    renderPage = () => RHtml(form([
        div([
            label("Username:"),
            input(name: "username", value: state.username),
            errorMsg(state.errors["username"])
        ]),
        div([
            label("Email:"),
            input(name: "email", value: state.email),
            errorMsg(state.errors["email"])
        ]),
        button("Submit", dataAction: "submit")
    ]))

    onAction = (action, params) => match action {
        "submit" -> {
            state.username = params["username"]
            state.email = params["email"]
            if validate(state) then {
                saveUser(state.username, state.email)
            }
        }
        _ -> ()
    }

    (renderPage, onAction)
}
```

## Parent/Child Introspection

```nostos
reactive Parent = { children: List[Child] }
reactive Child = { name: String, value: Int }

main() = {
    parent = Parent { children: [] }
    child1 = Child { name: "first", value: 10 }
    child2 = Child { name: "second", value: 20 }

    parent.children = [child1, child2]

    # Check what holds a reference to child1
    parents = child1.parents
    println("child1 is held by " ++ show(parents.length()) ++ " parent(s)")

    # Get all children
    allChildren = parent.children
    println("Parent has " ++ show(allChildren.length()) ++ " children")
}
```

## Computed Values Pattern

```nostos
reactive Cart = { items: List[CartItem] }
reactive CartItem = { name: String, price: Float, quantity: Int }

# Computed total (not reactive, but derived)
cartTotal(cart: Cart) -> Float = {
    cart.items.fold(0.0, (sum, item) => {
        sum + (item.price * item.quantity.toFloat())
    })
}

# Re-render when items change
cart.onChange("items", (_, _) => {
    total = cartTotal(cart)
    updateTotalDisplay(total)
})
```

## Debounced Updates

```nostos
reactive SearchState = { query: String, results: List[String] }

# Debounce search to avoid too many API calls
debounce(ms: Int, fn: () -> ()) = {
    mvar timer: Option[Pid] = None

    () => {
        # Cancel previous timer
        match timer {
            Some(pid) -> send(pid, "cancel")
            None -> ()
        }

        # Start new timer
        me = self()
        newTimer = spawn(() => {
            match receiveTimeout(ms) {
                None -> send(me, "fire")  # Timeout = execute
                Some("cancel") -> ()       # Cancelled
            }
        })
        timer = Some(newTimer)
    }
}

main() = {
    state = SearchState { query: "", results: [] }
    debouncedSearch = debounce(300, () => {
        state.results = searchApi(state.query)
    })

    state.onChange("query", (_, _) => debouncedSearch())
}
```

## Undo/Redo Pattern

```nostos
reactive Editor = { content: String }

type EditorHistory = {
    past: List[String],
    future: List[String]
}

mvar history: EditorHistory = EditorHistory { past: [], future: [] }

trackHistory(editor: Editor) = {
    editor.onChange("content", (old, new) => {
        history = EditorHistory {
            past: history.past ++ [old],
            future: []  # Clear redo stack on new edit
        }
    })
}

undo(editor: Editor) = {
    match history.past {
        [] -> ()
        [..rest, last] -> {
            current = editor.content
            editor.content = last  # This won't trigger onChange for history
            history = EditorHistory {
                past: rest,
                future: [current] ++ history.future
            }
        }
    }
}

redo(editor: Editor) = {
    match history.future {
        [] -> ()
        [next, ..rest] -> {
            current = editor.content
            editor.content = next
            history = EditorHistory {
                past: history.past ++ [current],
                future: rest
            }
        }
    }
}
```
