# d1c - Type-Safe D1 Query Generation

## What is d1c?

`hb-d1c` generates type-safe Rust functions from SQL queries. Write SQL → get Rust.

## Critical Rules

1. **NEVER edit files in `apps/backend/src/d1c/`** - They are machine-generated
2. **NEVER run `d1c generate`** - Assume `d1c watch` is running in the background
3. **Always use named parameters** (`:id`) - Never positional (`?1`)

## Workflow

1. Write/edit SQL in `apps/backend/db/queries/*.sql`
2. d1c automatically regenerates `apps/backend/src/d1c/*.rs`
3. Import and use the generated functions

## Query Format

```sql
-- name: FunctionName :cardinality
-- params: custom_id CustomType  (optional - for newtype wrappers)
SELECT ... WHERE id = :id;
```

### Cardinalities

| Cardinality | Returns | Use Case |
|-------------|---------|----------|
| `:one` | `Result<Option<Row>>` | Single row by ID, INSERT RETURNING |
| `:many` | `Result<Vec<Row>>` | Lists, filtered queries |
| `:exec` | `Result<()>` | UPDATE/DELETE without RETURNING |
| `:scalar` | `Result<Option<T>>` | COUNT, SUM, EXISTS |

### Named Parameters

```sql
-- Good
SELECT * FROM users WHERE id = :id AND org_id = :org_id;

-- Bad (never do this)
SELECT * FROM users WHERE id = ?1 AND org_id = ?2;
```

## Example

**SQL** (`apps/backend/db/queries/monitors.sql`):
```sql
-- name: get_monitor_by_id :one
SELECT * FROM monitors WHERE id = :id AND org_id = :org_id;

-- name: list_monitors_for_org :many
SELECT * FROM monitors WHERE org_id = :org_id ORDER BY created_at DESC;

-- name: delete_monitor :exec
DELETE FROM monitors WHERE id = :id AND org_id = :org_id;
```

**Usage** (Rust):
```rust
use crate::d1c::monitors::*;

let monitor = get_monitor_by_id(&d1, &monitor_id, &org_id).await?;
let monitors = list_monitors_for_org(&d1, &org_id).await?;
delete_monitor(&d1, &monitor_id, &org_id).await?;
```

## Custom Types with `-- params:`

For newtype wrappers or when inference fails:

```sql
-- name: get_user :one
-- params: user_id UserId, org_id OrgId
SELECT * FROM users WHERE id = :user_id AND org_id = :org_id;
```

## Batching with `:stmt`

Generate prepared statements for `d1.batch()`:

```sql
-- name: create_log :exec :stmt
INSERT INTO logs (id, message) VALUES (:id, :message);
```

```rust
let stmt1 = create_log_stmt(&d1, "1", "msg1")?;
let stmt2 = create_log_stmt(&d1, "2", "msg2")?;
d1.batch(vec![stmt1, stmt2]).await?;
```

## File Locations

- **SQL queries**: `apps/backend/db/queries/*.sql`
- **Generated code**: `apps/backend/src/d1c/*.rs` (don't edit!)
- **Schema**: `apps/backend/db/queries/schema.sql`
- **Full docs**: `crates/hb-d1c/QUERY_FORMAT.md`
