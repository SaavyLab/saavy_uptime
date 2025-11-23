# Observability Strategy – Tracing + Analytics Engine

We treat observability as a product feature, not an ops afterthought. The same instrumentation that keeps Saavy Uptime healthy powers the end‑user experience (execution DAG, geo map, cost dashboard, incident replay).

> Update: Cloudflare Workers Observability now provides built-in tracing/logging across fetch/bindings/handlers (`observability.traces` / `observability.logs` in `wrangler.toml`). The AE-first approach below is kept for legacy notes and product telemetry experiments.

## Architecture

```
Application Code (Axum API, Ticker DO, Dispatch Runner)
├─ #[tracing::instrument] spans + structured events
└────► AnalyticsEngineSubscriber (custom tracing layer)
         ├─ Captures span metadata (trace_id, monitor_id, colo, duration_ms, status)
         ├─ Batches writes
         └─ Sends to Analytics Engine via Workers binding

Analytics Engine dataset (schema: trace_id, span_id, parent_span_id, doubles, blobs)
├─ Grafana connector (operational dashboards)
└─ Product UI (execution DAG, geo view, cost prediction, incident replay)
```

## Why “tracing + AE” instead of OpenTelemetry?

| Aspect | OTEL | tracing + AE |
| --- | --- | --- |
| Target architecture | Polyglot microservices, separate collector | Rust monorepo on Workers |
| WASM support | minimal / experimental | Works today (custom subscriber) |
| Dependencies | Collector + external backend | AE already bundled with Workers |
| Query language | TraceQL | SQL |
| Cost | Pay external SaaS | Included in Workers bill |

Workers + Rust + AE gives us full-stack observability with a couple hundred LOC.

## Instrumentation plan

1. Annotate critical paths with `#[tracing::instrument]` (ticker alarm, D1 queries, dispatch runner, monitor execution).
2. Create an `AnalyticsEngineLayer` (tracing subscriber) that:
   - Captures `on_new_span`, `on_record`, `on_event`.
   - Serializes span data into AE rows (`trace_id`, `span_id`, `parent_span_id`, `span_name`, `duration_ms`, `monitor_id`, `target`, `cf.colo`, `status`, `error`).
   - Buffers writes per-worker isolate and flushes before DROP (or per `Timer`).
   - Uses the Workers AE HTTP API (until a workers-rs binding exists).
3. Expose `observability::init()` in the Worker entrypoint to install the subscriber during startup.
4. Define a shared schema in code + docs so AE queries/Dashboards stay stable.

## AE schema draft

| Column | Type | Notes |
| --- | --- | --- |
| `trace_id` | index | propagate via `tracing::Span::current().id()` |
| `span_id` | blob | unique per span |
| `parent_span_id` | blob | for DAG reconstruction |
| `span_name` | blob | e.g. `ticker.run_tick`, `dispatch.http`, `d1.insert_heartbeat` |
| `timestamp` | auto | inserted by AE |
| `duration_ms` | double | derived from span enter/exit |
| `monitor_id` | blob | optional per span (WHERE filter) |
| `org_id` | blob | same |
| `colo` | blob | from `cf.colo` or env |
| `status` | blob | success / error / http code |
| `rtt_ms` | double | for dispatch spans |
| `worker_invocations` | double | e.g. 1 per span for cost prediction |

## Queries we care about

- **Execution DAG**: `SELECT * FROM traces WHERE trace_id = ? ORDER BY timestamp`
- **Ticker health**: `SELECT AVG(duration_ms) FROM traces WHERE span_name = 'ticker.alarm' AND timestamp > NOW() - INTERVAL '1 hour'`
- **Geo distribution**: `SELECT colo, COUNT(*), AVG(rtt_ms) FROM traces WHERE monitor_id = ? GROUP BY colo`
- **Cost predictor**: `SELECT SUM(worker_invocations) FROM traces WHERE timestamp > NOW() - INTERVAL '30 day'`
- **Incident replay**: `SELECT * FROM traces WHERE monitor_id = ? AND timestamp BETWEEN ? AND ?`

## Grafana dashboards

Ship sample dashboards under `grafana/`:

1. `operational-overview.json` – DO alarm latency, dispatch rates, error budget.
2. `performance-deep-dive.json` – p50/p95/p99 per span, D1/AE latencies.
3. `incident-response.json` – live incident timeline, affected monitors, recent traces.

Users connect Grafana to AE, import JSON dashboards, instant observability.

## Open-source roadmap

Phase 1–2: keep subscriber in repo (`apps/backend/src/observability/ae_layer.rs`) until API stabilizes.  
Phase 3+: consider extracting to `tracing-analytics-engine` crate if other teams ask for it (signals: GitHub issues, Cloudflare DevRel interest).  
Long-term: pitch the pattern to Cloudflare (blog post, docs contribution, example repo).

## AE API status

Workers don’t have first-class AE bindings. Reads/writes use the REST API (`/client/v4/accounts/{ACCOUNT_ID}/analytics_engine/sql`) with API tokens. We may eventually PR bindings into workers-rs (high leverage OSS contribution).

## TL;DR

One instrumentation pass (tracing → AE) powers:

- Ops debugging (Grafana)
- Product features (DAG, geo, cost, incident replay)
- Marketing story (“Cloudflare-native observability”)

Stay Cloudflare-native, keep everything in Workers + D1 + AE + R2. Observability is the differentiator, not a cost center.
