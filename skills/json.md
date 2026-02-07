# JSON in Nostos

## Parsing JSON

```nostos
# Parse JSON string to dynamic value
data = json.parse('{"name": "Alice", "age": 30}')

# Access fields with bracket syntax
name = data["name"]     # "Alice"
age = data["age"]       # 30

# Nested access
config = json.parse('{"server": {"host": "localhost", "port": 8080}}')
host = config["server"]["host"]  # "localhost"

# Arrays
items = json.parse('[1, 2, 3, 4, 5]')
first = items[0]        # 1
```

## Generating JSON

```nostos
# Convert map to JSON string
data = %{"name": "Bob", "active": true, "scores": [95, 87, 92]}
jsonStr = json.stringify(data)
# '{"name":"Bob","active":true,"scores":[95,87,92]}'

# Pretty print
prettyJson = json.stringifyPretty(data)
```

## Type-Safe JSON with Records

```nostos
use stdlib.json.*

type User = { name: String, email: String, age: Int }

# Parse JSON directly to typed record
jsonStr = '{"name": "Alice", "email": "alice@example.com", "age": 30}'
user: User = fromJson[User](jsonStr)

println(user.name)   # "Alice"
println(user.age)    # 30

# Convert record to JSON
userJson = toJson(user)
```

## Handling Variants

```nostos
type Status = Active | Inactive(String) | Pending { since: Int }

# Variant serialization
statusJson = toJson(Active)           # {"variant": "Active"}
statusJson = toJson(Inactive("left")) # {"variant": "Inactive", "value": "left"}
statusJson = toJson(Pending { since: 1234 })
# {"variant": "Pending", "fields": {"since": 1234}}

# Parse back to variant
status: Status = fromJson[Status]('{"variant": "Active"}')
```

## Parameterized Types (List, Option, Map)

`fromJsonValue` handles builtin parameterized types in record fields:

```nostos
use stdlib.json.{jsonParse, fromJsonValue}

# List[T] - JSON arrays become typed lists
type Item = { name: String, price: Float }
type Order = { items: List[Item], total: Float }

order: Order = fromJsonValue("Order", jsonParse('{"items": [{"name": "Widget", "price": 9.99}], "total": 9.99}'))
head(order.items).name  # "Widget"

# Option[T] - null becomes None, values become Some
type Config = { name: String, desc: Option[String] }

c1: Config = fromJsonValue("Config", jsonParse('{"name": "app", "desc": "my app"}'))
# c1.desc = Some("my app")

c2: Config = fromJsonValue("Config", jsonParse('{"name": "app", "desc": null}'))
# c2.desc = None

# Map[K, V] - JSON objects become typed maps
type Settings = { config: Map[String, Int] }

s: Settings = fromJsonValue("Settings", jsonParse('{"config": {"width": 100, "height": 200}}'))
Map.get(s.config, "width")  # 100

# Nested parameterized types work too
type Matrix = { data: List[List[Int]] }
# List[Option[String]], Map[String, List[Item]], etc.

# Generic types with compound fields
type Wrapper[T] = { value: T, items: List[T] }
w = fromJsonValue("Wrapper[Int]", jsonParse('{"value": 42, "items": [1, 2, 3]}'))
# Type param T is substituted in List[T] -> List[Int]
```

## Nested Records

```nostos
type Address = { street: String, city: String }
type Person = { name: String, address: Address }

jsonStr = '''
{
    "name": "Alice",
    "address": {
        "street": "123 Main St",
        "city": "Oslo"
    }
}
'''

person: Person = fromJson[Person](jsonStr)
println(person.address.city)  # "Oslo"
```

## Lists of Records

```nostos
type Product = { id: Int, name: String, price: Float }

jsonStr = '''
[
    {"id": 1, "name": "Widget", "price": 9.99},
    {"id": 2, "name": "Gadget", "price": 19.99}
]
'''

products: List[Product] = fromJson[List[Product]](jsonStr)
products.each(p => println(p.name ++ ": $" ++ show(p.price)))
```

## Optional Fields

```nostos
type Config = {
    host: String,
    port: Int,
    debug: Option[Bool]  # Optional field
}

# JSON without optional field
config1: Config = fromJson[Config]('{"host": "localhost", "port": 8080}')
config1.debug  # None

# JSON with optional field
config2: Config = fromJson[Config]('{"host": "localhost", "port": 8080, "debug": true}')
config2.debug  # Some(true)
```

## Error Handling

```nostos
# Safe parsing
result = try {
    data = json.parse(inputStr)
    Some(data)
} catch e {
    println("Parse error: " ++ e)
    None
}

# Type-safe parsing with Result
parseUser(jsonStr: String) -> Result[User, String] = {
    try {
        Ok(fromJson[User](jsonStr))
    } catch e {
        Err("Failed to parse user: " ++ e)
    }
}
```

## Dynamic JSON Access

```nostos
# Check field existence
data = json.parse('{"name": "Alice"}')
hasEmail = data.contains("email")  # false

# Get with default
email = data.get("email").getOrElse("no-email@example.com")

# Iterate over object fields
data = json.parse('{"a": 1, "b": 2, "c": 3}')
data.keys().each(key => println(key ++ ": " ++ show(data[key])))
```

## JSON Path-like Access

```nostos
# Deep access helper
getPath(data, path: List[String]) = {
    path.fold(data, (current, key) => current[key])
}

config = json.parse('''
{
    "database": {
        "connection": {
            "host": "localhost"
        }
    }
}
''')

host = getPath(config, ["database", "connection", "host"])
# "localhost"
```

## Building JSON Dynamically

```nostos
# Start with empty map
builder = %{}

# Add fields
builder["name"] = "Alice"
builder["age"] = 30
builder["tags"] = ["developer", "nostos"]

# Convert to JSON string
json.stringify(builder)
```

## JSON API Response Pattern

```nostos
type ApiResponse[T] = {
    success: Bool,
    data: Option[T],
    error: Option[String]
}

successResponse(data) = ApiResponse {
    success: true,
    data: Some(data),
    error: None
}

errorResponse(msg: String) = ApiResponse {
    success: false,
    data: None,
    error: Some(msg)
}

# In handler
handler(req) = {
    result = try {
        data = processRequest(req)
        successResponse(data)
    } catch e {
        errorResponse(e)
    }
    respondJson(req, toJson(result))
}
```

## JSON Transformation

```nostos
# Transform JSON structure
transformUser(data) = %{
    "fullName": data["firstName"] ++ " " ++ data["lastName"],
    "contact": %{
        "email": data["email"],
        "phone": data["phone"]
    }
}

input = json.parse('{"firstName": "Alice", "lastName": "Smith", "email": "a@b.com", "phone": "123"}')
output = transformUser(input)
json.stringify(output)
```

## Round-Trip Testing

```nostos
# Verify JSON encode/decode preserves data
testRoundTrip(value: T) -> Bool = {
    encoded = toJson(value)
    decoded: T = fromJson[T](encoded)
    decoded == value
}

# Test
user = User { name: "Alice", email: "a@b.com", age: 30 }
assert(testRoundTrip(user))
```

## JSON with Reflection

```nostos
# Get type info at runtime
info = typeInfo("User")
fields = info["fields"]  # List of field definitions

# Dynamic record construction from JSON
buildFromJson(typeName: String, jsonStr: String) = {
    data = json.parse(jsonStr)
    makeRecordByName(typeName, data)
}
```
