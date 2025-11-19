# hb-tracing

**Distributed tracing for Cloudflare Workers.**

`hb-tracing` bridges Rust's battle-tested tracing ecosystem with Cloudflare's serverless platform. It captures spans, correlates them across distributed systems, and writes them to Analytics Engine via background Queues.

Part of the [**hb** stack](../hb-README.md).

---

## The Problem

Debugging Workers is hard. `console.log` is unstructured. Analytics Engine is powerful but manual. Writing to it synchronously adds latency to your user's request.

## The Solution

`hb-tracing` instruments your code once with standard `#[tracing::instrument]` macros.

- **Zero Latency:** Spans are buffered in memory and flushed to a Queue after the HTTP response is sent (using `ctx.waitUntil`).
- **High Reliability:** The Queue consumer handles batching, retries, and writes to Analytics Engine in bulk.
- **Deep Visibility:** Distributed trace IDs correlate logs across Workers, Durable Objects, and external services.

---

## Quick Start

### 1. Install

```toml
[dependencies]
hb-tracing = { path = "../hb-tracing" }
tracing = "0.1"
worker = "0.4"
```

### 2. Configure `wrangler.toml`

You need an Analytics Engine dataset and a Queue to decouple writing from serving.

```toml
# The Dataset
[[analytics_engine_datasets]]
binding = "OBSERVABILITY"
dataset = "observability_prod"

# The Queue (Producer & Consumer)
[[queues.producers]]
queue = "tracing-spans"
binding = "TRACE_QUEUE"

[[queues.consumers]]
queue = "tracing-spans"
max_batch_size = 100
max_batch_timeout = 5
```

### 3. Instrument Your Worker

In your API Worker (`src/lib.rs`): Initialize the layer, run your logic, and flush the buffer in the background.

```rust
use worker::*;
use hb_tracing::{buffer_layer, ConsoleLayer};
use tracing_subscriber::prelude::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    // 1. Setup Tracing
    let (layer, guard) = buffer_layer();
    let subscriber = tracing_subscriber::registry()
        .with(ConsoleLayer) // Logs to "wrangler tail"
        .with(layer);       // Buffers for Analytics Engine

    // 2. Run Request
    let response = tracing::subscriber::with_default(subscriber, || async {
        handle_request(req, &env).await
    }).await?;

    // 3. Flush to Queue (Background)
    let queue = env.queue("TRACE_QUEUE")?;
    ctx.wait_until(async move {
        if let Err(e) = guard.flush(&queue).await {
            console_error!("Failed to flush traces: {e}");
        }
    });

    Ok(response)
}
```

In your Queue Consumer (`src/lib.rs` or `src/tracing_consumer.rs`): This worker (or entrypoint) wakes up to process the batch and write to AE.

```rust
use hb_tracing::SpanEvent;

#[event(queue)]
pub async fn queue(batch: MessageBatch<SpanEvent>, env: Env, _ctx: Context) -> Result<()> {
    let ae = env.analytics_engine("OBSERVABILITY")?;
    hb_tracing::handle_batch(batch, &ae)?;
    Ok(())
}
```

---

## Usage

Use standard tracing instrumentation.

```rust
#[tracing::instrument(skip(d1), fields(user_id))]
pub async fn create_monitor(d1: &D1Database, user_id: &str) -> Result<()> {
    tracing::info!("Creating monitor for user");
    
    // Custom fields are searchable in Analytics Engine
    tracing::Span::current().record("user_id", user_id);
    
    // Nested spans are automatically correlated
    let _ = db_query(d1).await?;
    
    Ok(())
}
```

---

## Data Schema

We write a structured schema to Analytics Engine optimized for Grafana.

| Field | AE Column | Type | Description |
|-------|-----------|------|-------------|
| `trace_id` | `index1` | String | Correlates spans across a request tree. |
| `span_id` | `blob1` | String | Unique ID of this operation. |
| `parent_id` | `blob2` | String | ID of the caller (or "root"). |
| `name` | `blob3` | String | Function name (e.g., `create_monitor`). |
| `target` | `blob4` | String | Module path (e.g., `backend::monitors`). |
| `level` | `blob5` | String | INFO, ERROR, etc. |
| `fields` | `blob6` | String | JSON object of all captured arguments/fields. |
| `duration_ms` | `double1` | Number | Execution time in milliseconds. |
| `timestamp` | `double2` | Number | Start time (Unix epoch ms). |

---

## Grafana Queries

### Find Slowest Operations (P99)

```sql
SELECT
  blob3 as operation,
  quantile(0.99)(double1) as p99_latency
FROM observability_prod
WHERE timestamp > NOW() - INTERVAL '1' HOUR
GROUP BY operation
ORDER BY p99_latency DESC
```

### Trace a Request

```sql
SELECT
  blob3 as name,
  double1 as duration_ms,
  blob6 as fields
FROM observability_prod
WHERE index1 = '$TRACE_ID'
ORDER BY double2 ASC
```

---

## License

MIT