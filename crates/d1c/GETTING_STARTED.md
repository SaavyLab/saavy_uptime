# Getting Started with d1c

This guide will help you set up **d1c** in your Cloudflare Worker project.

## Prerequisites

1.  **Rust & Cargo**: You should have a working Rust environment.
2.  **Wrangler**: Cloudflare's CLI tool (`npm install -g wrangler`).
3.  **A D1 Database**: Created via `wrangler d1 create <name>`.

---

## 1. Installation

Install the `d1c` CLI tool:

```bash
cargo install d1c
```

---

## 2. Setup

Navigate to your Worker's crate directory (where `wrangler.toml` is) and run:

```bash
d1c init
```

This interactive command will:
1.  Detect your D1 database configuration in `wrangler.toml`.
2.  Ask where your migrations and queries are located.
3.  Create a `d1c.toml` configuration file.
4.  Generate a sample query file (`example.sql`) if your query directory is empty.

---

## 3. The Workflow

d1c follows a simple cycle: **Migrate → Query → Generate**.

### Step 3a: Write Migrations
Create your D1 migrations as usual. d1c uses these to understand your schema.

```sql
-- migrations/0001_init.sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL,
  active BOOLEAN NOT NULL DEFAULT FALSE
);
```

**Important**: Ensure you apply them locally so d1c can introspect them:
```bash
wrangler d1 migrations apply DB --local
```

### Step 3b: Write Queries
Create `.sql` files in your queries directory (e.g., `db/queries/users.sql`). Use the `-- name:` header to define the function name and return type.

```sql
-- name: GetUser :one
SELECT id, email, active FROM users WHERE id = :id;

-- name: CreateUser :one
INSERT INTO users (id, email) VALUES (:id, :email) RETURNING *;
```

**Supported Cardinalities:**
- `:one`    -> Returns `Result<Option<Row>>`
- `:many`   -> Returns `Result<Vec<Row>>`
- `:exec`   -> Returns `Result<()>` (no return value)
- `:scalar` -> Returns `Result<Option<T>>` (single primitive value)

### Step 3c: Generate Code
Run the generator:

```bash
d1c generate
```

This creates a Rust file (default: `src/d1c/d1c.rs`) containing type-safe functions for your queries.

> **Pro Tip**: Run `d1c watch` in a separate terminal to automatically regenerate code whenever you save a `.sql` file.

---

## 4. Using the Generated Code

Import the module in your Worker and use the functions. They are `async` and take the `D1Database` as the first argument.

```rust
// src/lib.rs or src/main.rs
mod d1c; // matching the directory you generated into
use crate::d1c::d1c::*; // import generated functions

#[worker::event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let d1 = env.d1("DB")?;
    
    // 1. Fetch a user (Type-safe!)
    let user = get_user(&d1, "user_123").await?;
    
    if let Some(u) = user {
        console_log!("Found user: {}", u.email);
    }

    // 2. Create a user (Compiler enforces correct arguments!)
    create_user(&d1, "user_456", "new@example.com").await?;

    Response::ok("Done")
}
```

---

## Advanced Usage

### Tracing / Observability
If you use `tracing`, you can auto-instrument all database queries.
1.  Set `instrument_by_default = true` in `d1c.toml`.
2.  Generated functions will have `#[tracing::instrument(name = "d1c.query_name")]`.

To skip sensitive fields (like passwords) from logs:
```sql
-- name: Login :one
-- instrument: skip(password)
SELECT * FROM users WHERE email = :email AND password = :password;
```

### Explicit Parameter Types
If type inference fails or you want to use a custom type (like a NewType wrapper), use the `-- params:` header:

```sql
-- name: GetUserBalance :one
-- params: user_id UserId
SELECT balance FROM accounts WHERE user_id = :user_id;
```
*Note: You must ensure `UserId` is in scope in your Rust code.*
