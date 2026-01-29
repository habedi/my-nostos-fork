# WebSockets in Nostos

## WebSocket Server

```nostos
use stdlib.server.*

handler(req) = {
    if WebSocket.isUpgrade(req) then {
        ws = WebSocket.accept(req.id)
        handleWebSocket(ws)
    } else {
        respondText(req, "WebSocket endpoint")
    }
}

handleWebSocket(ws) = {
    msg = WebSocket.recv(ws)
    WebSocket.send(ws, "Echo: " ++ msg)
    handleWebSocket(ws)  # Loop for more messages
}

main() = serve(8080, handler)
```

## WebSocket Client

```nostos
# Connect to WebSocket server
ws = WebSocket.connect("wss://echo.websocket.org")

# Send message
WebSocket.send(ws, "Hello, WebSocket!")

# Receive message
response = WebSocket.recv(ws)
println("Got: " ++ response)

# Close connection
WebSocket.close(ws)
```

## Message Types

```nostos
# Text messages (default)
WebSocket.send(ws, "Hello")
msg = WebSocket.recv(ws)  # String

# JSON messages
data = %{"type": "chat", "text": "Hello"}
WebSocket.send(ws, json.stringify(data))

received = json.parse(WebSocket.recv(ws))
msgType = received["type"]
```

## Chat Server Pattern

```nostos
mvar clients: List[WebSocket] = []

broadcast(message: String) = {
    clients.each(ws => {
        try { WebSocket.send(ws, message) }
        catch _ { () }  # Ignore send errors
    })
}

handleClient(ws) = {
    # Add to client list
    clients = clients ++ [ws]

    loop() = {
        try {
            msg = WebSocket.recv(ws)
            broadcast(msg)
            loop()
        } catch _ {
            # Remove on disconnect
            clients = clients.filter(c => c != ws)
        }
    }
    loop()
}

handler(req) = {
    if WebSocket.isUpgrade(req) then {
        ws = WebSocket.accept(req.id)
        spawn(() => handleClient(ws))
    } else {
        respondText(req, "Chat server")
    }
}

main() = serve(8080, handler)
```

## Typed Message Protocol

```nostos
type ClientMessage =
    | Join { username: String }
    | Chat { text: String }
    | Leave

type ServerMessage =
    | Welcome { users: List[String] }
    | UserJoined { username: String }
    | Message { from: String, text: String }
    | UserLeft { username: String }

parseClientMessage(raw: String) -> ClientMessage = {
    data = json.parse(raw)
    match data["type"] {
        "join" -> Join { username: data["username"] }
        "chat" -> Chat { text: data["text"] }
        "leave" -> Leave
    }
}

sendServerMessage(ws, msg: ServerMessage) = {
    json = match msg {
        Welcome { users } -> %{"type": "welcome", "users": users}
        UserJoined { username } -> %{"type": "joined", "username": username}
        Message { from, text } -> %{"type": "message", "from": from, "text": text}
        UserLeft { username } -> %{"type": "left", "username": username}
    }
    WebSocket.send(ws, json.stringify(json))
}
```

## Room-Based Chat

```nostos
type Room = { name: String, clients: List[WebSocket] }

mvar rooms: Map[String, Room] = %{}

joinRoom(roomName: String, ws: WebSocket) = {
    room = rooms[roomName].getOrElse(Room { name: roomName, clients: [] })
    updatedRoom = Room { name: roomName, clients: room.clients ++ [ws] }
    rooms[roomName] = updatedRoom
}

broadcastToRoom(roomName: String, message: String) = {
    match rooms[roomName] {
        Some(room) -> room.clients.each(ws => WebSocket.send(ws, message))
        None -> ()
    }
}

leaveRoom(roomName: String, ws: WebSocket) = {
    match rooms[roomName] {
        Some(room) -> {
            updated = Room { name: roomName, clients: room.clients.filter(c => c != ws) }
            rooms[roomName] = updated
        }
        None -> ()
    }
}
```

## Ping/Pong Heartbeat

```nostos
handleWithHeartbeat(ws) = {
    lastPing = currentTimeMillis()

    # Spawn heartbeat checker
    spawn(() => {
        while true {
            sleep(30000)  # Check every 30s
            if currentTimeMillis() - lastPing > 60000 then {
                WebSocket.close(ws)
                break
            }
        }
    })

    loop() = {
        msg = WebSocket.recv(ws)
        if msg == "ping" then {
            lastPing = currentTimeMillis()
            WebSocket.send(ws, "pong")
        } else {
            handleMessage(ws, msg)
        }
        loop()
    }
    loop()
}
```

## Binary Data

```nostos
# Send binary data as base64
sendBinary(ws, data: List[Int]) = {
    encoded = base64Encode(data)
    WebSocket.send(ws, encoded)
}

# Receive and decode
receiveBinary(ws) -> List[Int] = {
    encoded = WebSocket.recv(ws)
    base64Decode(encoded)
}
```

## Error Handling

```nostos
handleWebSocket(ws) = {
    try {
        loop() = {
            msg = WebSocket.recv(ws)
            response = processMessage(msg)
            WebSocket.send(ws, response)
            loop()
        }
        loop()
    } catch {
        "connection closed" -> println("Client disconnected")
        e -> println("WebSocket error: " ++ e)
    }
}
```

## Reconnection (Client)

```nostos
connectWithRetry(url: String, maxRetries: Int) -> Option[WebSocket] = {
    attempt(retries) = {
        if retries <= 0 then None
        else {
            try {
                ws = WebSocket.connect(url)
                Some(ws)
            } catch _ {
                println("Connection failed, retrying in 1s...")
                sleep(1000)
                attempt(retries - 1)
            }
        }
    }
    attempt(maxRetries)
}
```

## Broadcast with Sender Filtering

```nostos
mvar clientMap: Map[Int, WebSocket] = %{}
mvar nextId: Int = 0

addClient(ws: WebSocket) -> Int = {
    id = nextId
    nextId = nextId + 1
    clientMap[id] = ws
    id
}

broadcastExcept(senderId: Int, message: String) = {
    clientMap.entries().each((id, ws) => {
        if id != senderId then {
            try { WebSocket.send(ws, message) }
            catch _ { clientMap = clientMap.remove(id) }
        }
    })
}
```
