# Database Operations in Nostos

## PostgreSQL Connection

```nostos
# Connect to PostgreSQL
conn = Pg.connect("host=localhost dbname=mydb user=postgres password=secret")

# Or with more options
conn = Pg.connect("host=localhost port=5432 dbname=mydb user=postgres password=secret sslmode=prefer")

# Close connection when done
Pg.close(conn)
```

## Basic Queries

```nostos
# Query returns list of tuples
rows = Pg.query(conn, "SELECT name, email FROM users", [])

# Access columns positionally
rows.map(row => println(row.0 ++ ": " ++ row.1))

# Parameterized queries (prevent SQL injection)
rows = Pg.query(conn, "SELECT * FROM users WHERE active = $1 AND age > $2", [true, 18])

# Single row access
firstRow = head(rows)
name = firstRow.0
email = firstRow.1
```

## Execute (Non-Query Operations)

```nostos
# INSERT, UPDATE, DELETE don't return rows
Pg.execute(conn, "INSERT INTO users (name, email) VALUES ($1, $2)", ["Alice", "alice@example.com"])

Pg.execute(conn, "UPDATE users SET active = $1 WHERE id = $2", [true, 42])

Pg.execute(conn, "DELETE FROM users WHERE id = $1", [42])
```

## Typed Results with Introspection

Map query results to typed records using the `stdlib.db` module:

```nostos
use stdlib.db.{rowsToRecords, rowToRecord, queryAs}

type User = { id: Int, name: String, email: String }

main() = {
    conn = Pg.connect("host=localhost user=postgres password=postgres")

    # Query and map to typed records
    rows = Pg.query(conn, "SELECT id, name, email FROM users", [])
    users: List[User] = rowsToRecords("User", rows)

    # Now use field names instead of positional access
    users.map(u => println(u.name ++ " <" ++ u.email ++ ">"))

    # Filter by field
    active = users.filter(u => u.id > 10)

    # Map by field
    emails = users.map(u => u.email)

    Pg.close(conn)
}
```

**Important:** Column order in SELECT must match field order in the type definition.

## Type Conversions

PostgreSQL types map to Nostos types:

| PostgreSQL | Nostos |
|------------|--------|
| INTEGER, BIGINT | Int |
| REAL, DOUBLE | Float |
| TEXT, VARCHAR | String |
| BOOLEAN | Bool |
| JSON, JSONB | String (parse with jsonParse) |

## Transactions

```nostos
# Begin transaction
Pg.execute(conn, "BEGIN", [])

# Do work
Pg.execute(conn, "INSERT INTO orders (user_id, amount) VALUES ($1, $2)", [1, 100])
Pg.execute(conn, "UPDATE users SET balance = balance - $1 WHERE id = $2", [100, 1])

# Commit (or ROLLBACK on error)
Pg.execute(conn, "COMMIT", [])
```

## Connection Pooling

For production apps, use a connection pool:

```nostos
use stdlib.pool.*

# Create pool with max 10 connections
pool = Pool.create(10, () => Pg.connect("host=localhost user=postgres password=postgres"))

# Get connection from pool
conn = pool.acquire()

# Use connection
rows = Pg.query(conn, "SELECT * FROM users", [])

# Return to pool
pool.release(conn)
```

## Error Handling

```nostos
main() = {
    result = try {
        conn = Pg.connect("host=localhost user=postgres password=wrong")
        Pg.query(conn, "SELECT * FROM users", [])
    } catch e {
        println("Database error: " ++ e)
        []  # Return empty list on error
    }
    result
}
```

## Prepared Statements

For repeated queries, prepare once and execute many:

```nostos
# Prepare statement
Pg.execute(conn, "PREPARE get_user AS SELECT * FROM users WHERE id = $1", [])

# Execute prepared statement multiple times
user1 = Pg.query(conn, "EXECUTE get_user(1)", [])
user2 = Pg.query(conn, "EXECUTE get_user(2)", [])
user3 = Pg.query(conn, "EXECUTE get_user(3)", [])

# Deallocate when done
Pg.execute(conn, "DEALLOCATE get_user", [])
```

## Listen/Notify (Pub/Sub)

```nostos
# In subscriber process
conn1 = Pg.connect("host=localhost user=postgres password=postgres")
Pg.execute(conn1, "LISTEN my_channel", [])

# Wait for notification (blocking)
notification = Pg.waitForNotification(conn1)
println("Got: " ++ notification)

# In publisher process
conn2 = Pg.connect("host=localhost user=postgres password=postgres")
Pg.execute(conn2, "NOTIFY my_channel, 'hello'", [])
```
