# Modules in Nostos

## File Structure

```
project/
├── main.nos          # Entry point with main()
├── utils.nos         # Module file
├── models/
│   ├── user.nos      # Nested module
│   └── order.nos
└── nostos.toml       # Optional project config
```

## Importing Modules

```nostos
# Import a module (file in same directory)
import utils

# Use qualified names
utils.helper()
utils.Config

# Import nested module
import models.user

models.user.createUser("Alice")
```

## Using Imports

```nostos
# Import and bring names into scope
import utils
use utils.*             # Import all public names

helper()                # Can use directly now
Config                  # Type is available

# Selective import
use utils.{helper, Config}

# Alias imports
import models.user as u
u.createUser("Alice")
```

## Public vs Private

```nostos
# In utils.nos:

# Public function (accessible from other modules)
pub helper() = "I'm public"

# Public type
pub type Config = { name: String, value: Int }

# Private function (only in this module)
internalHelper() = "I'm private"

# Private type
type InternalState = { data: List[Int] }
```

## Exporting

```nostos
# Everything marked `pub` is exported

# Public function
pub greet(name: String) = "Hello, " ++ name

# Public type
pub type User = { name: String, email: String }

# Public constant
pub const MAX_USERS = 100

# Public trait
pub trait Serializable
    serialize(self) -> String
end
```

## Module Example

```nostos
# math_utils.nos

pub const PI = 3.14159

pub square(x: Float) = x * x

pub cube(x: Float) = x * x * x

pub circleArea(radius: Float) = PI * square(radius)

# Private helper
clamp(x, min, max) = {
    if x < min then min
    else if x > max then max
    else x
}

pub clampedSquare(x: Float) = square(clamp(x, 0.0, 100.0))
```

```nostos
# main.nos

import math_utils
use math_utils.*

main() = {
    println("PI = " ++ show(PI))
    println("5^2 = " ++ show(square(5.0)))
    println("Circle area = " ++ show(circleArea(3.0)))
}
```

## Standard Library Imports

```nostos
# The stdlib is auto-imported, but you can be explicit
import json
import io
import url
import logging

# Using stdlib
json.parse('{"key": "value"}')
io.readFile("data.txt")
```

## Circular Dependencies

```nostos
# Nostos handles circular imports
# But prefer avoiding them for clarity

# a.nos
import b
pub aFunc() = b.bFunc() + 1

# b.nos
import a
pub bFunc() = 10
# Avoid calling a.aFunc() here - would cause infinite loop
```

## Project Configuration (nostos.toml)

```toml
[project]
name = "myproject"
version = "0.1.0"

[dependencies]
# External packages (from GitHub)
nalgebra = { git = "https://github.com/user/nalgebra-nos" }

[extensions]
# Native extensions
glam = { version = "0.1.0" }
```

## Visibility Rules

```nostos
# Public items are accessible from anywhere
pub type PublicType = { data: Int }
pub publicFunc() = 42

# Private items only in same module
type PrivateType = { secret: String }
privateFunc() = "hidden"

# Trait implementations follow the type's visibility
PublicType: Show
    show(self) = show(self.data)
end
```

## Nested Modules

```nostos
# models/user.nos
pub type User = { id: Int, name: String }

pub createUser(name: String) -> User = User(0, name)

pub validateEmail(email: String) -> Bool =
    email.contains("@")
```

```nostos
# main.nos
import models.user
use models.user.{User, createUser}

main() = {
    user = createUser("Alice")
    println(user.name)
}
```

## Re-exporting

```nostos
# lib.nos - re-export from submodules

import models.user
import models.order
import utils

# Re-export for convenient access
pub use models.user.User
pub use models.order.Order
pub use utils.helper
```

```nostos
# main.nos
import lib
use lib.*

# Can now use User, Order, helper directly
user = User(1, "Alice")
```

## Module Initialization

```nostos
# Code at module level runs when imported
# Use for initialization

# config.nos
println("Config module loaded")  # Runs on import

pub const DEBUG = true
pub var connectionCount = 0      # Module-level mutable state
```

## Best Practices

```nostos
# 1. One module = one responsibility
# Good: user.nos for user-related code
# Bad: utils.nos with everything

# 2. Explicit exports
pub type User = ...     # Document public API

# 3. Use qualified names for clarity
import json
json.parse(data)        # Clear where parse comes from

# 4. Group related imports
import models.user
import models.order
import models.product

use models.user.User
use models.order.Order
use models.product.Product
```
