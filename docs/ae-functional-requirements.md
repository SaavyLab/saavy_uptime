# Analytics Engine Functional Requirements

Defines the user-facing requirements that dictate how we ingest, store, and query heartbeat data in Cloudflare Analytics Engine (AE). This doc bridges dashboard expectations → AE schema → backend query contracts so we can validate ingestion coverage and prioritize work.

## Personas & Core Questions

| Persona / View | Representative Questions | Required Insight | Priority |
| --- | --- | --- | --- |
| **Org admin – Fleet overview** | “Is the fleet healthy right now?”, “Which regions are failing?” | 24h/7d uptime %, failure counts by region/kind, list of failing monitors | P0 |
| **Monitor owner – Detail page** | “Did my monitor flap overnight?”, “What’s the latency trend and most recent failure context?” | Recent heartbeat timeline (≤100 rows), rolling uptime %, latency p50/p95, latest error payload | P0 |

Only these two personas are in scope for the current milestone. Incident response and cost telemetry use cases remain future backlog; capture extra fields opportunistically, but do not block monitor-health delivery on them.

## Data Requirements (What must land in AE)

### Heartbeat fact row (written by `internal::dispatch::persist_heartbeat_result`)

| Field | Purpose | Notes |
| --- | --- | --- |
| `ts` (Unix ms) | Ordering & window filters | Required for time windows (5m, 1h, 24h, 7d, 30d). |
| `org_id`, `monitor_id` | Authorization + grouping | Needed for per-org dashboards and monitor detail views. |
| `dispatch_id` | Incident correlation & dedup | Allows joining to D1 incidents and proving coverage. |
| `status_ok` (bool) | Uptime math | Drives uptime %, failure streaks. |
| `http_status` / `result_code` | Error taxonomy | Enables “most common failure” visual + filtering. |
| `latency_ms` | Latency trends | Used for percentile charts + SLA widgets. |
| `region` (`cf.colo` or logical region) | Geo heatmaps | Supports POP-level health & incident analysis. |
| `sample_rate` (float) or `sample_weight` | Accurate percentages under sampling | Required to avoid biased aggregates when AE writes are throttled. |
| `probe_kind` / `monitor_kind` | Filter by HTTP/TCP/etc. | Lets org admins slice health by monitor type. |
| `error_digest` (hash/category) | Bucket similar failures | Optional text; keep short to control storage. |

### Optional context rows (future backlog)

- **Incident snapshots** (`incident_id`, `started_at`, `resolved_at`, `consecutive_failures`) – nice-to-have for later MTTR/incident visuals; do not prioritize implementation work yet.
- **Cost telemetry** (`dispatch_duration_ms`, `ae_write_errors`) – only needed once we build the cost console.

If we can add these fields “for free” while touching ingestion, great; otherwise they stay deferred until the corresponding UI work restarts.

## Query & API Requirements (How AE data is consumed)

### 1. Monitor Detail Experience (P0)

| Metric / Widget | AE Query Shape | Backend Contract |
| --- | --- | --- |
| **Recent heartbeats table** (≤100 rows) | `SELECT * FROM heartbeats WHERE monitor_id = ? AND ts >= NOW()-interval ORDER BY ts DESC LIMIT 100` | `GET /api/monitors/:id/heartbeats?limit=100&window=24h` returns rows + `sample_rate`. Fallback to D1 if AE unavailable. |
| **Rolling uptime badge** | `SELECT SUM(CASE WHEN status_ok THEN sample_weight ELSE 0 END)/SUM(sample_weight)` over window | `GET /api/monitors/:id/uptime?window=24h` responds with uptime %, sample variance, window bounds. |
| **Latency sparkline (p50/p95)** | `SELECT approx_percentile(latency_ms, 0.5/0.95) GROUP BY bucket_5m(ts)` | Exposed via `GET /api/monitors/:id/latency?window=24h&bucket=5m` for chart consumption. |
| **Last failure summary** | `SELECT * FROM heartbeats WHERE monitor_id=? AND status_ok=false ORDER BY ts DESC LIMIT 1` | Included in heartbeat response so UI can show “Last failed 12m ago (HTTP 500 @ SJC)”. |

### 2. Fleet / Org Dashboard (P0)

| Widget | AE Query Shape | Notes |
| --- | --- | --- |
| **Org uptime cards (24h / 7d / 30d)** | `SELECT window, uptime_pct FROM (WINDOWED AGGREGATION)` | Aggregates grouped by window length; API returns array for cards. |
| **Failing monitors list** | `SELECT monitor_id, COUNT(*) AS failures_last_15m FROM heartbeats WHERE status_ok=false AND ts>=NOW()-15m GROUP BY monitor_id ORDER BY COUNT DESC LIMIT 10` | Drives “hot list” table. Requires join to monitor metadata from D1 after AE query. |
| **Region heatmap** | `SELECT region, SUM(sample_weight) AS checks, SUM(IF(status_ok, sample_weight, 0)) AS successes` | Returns success rate per region for choropleth or table. |
| **Latency distribution** | `SELECT bucket_15m(ts) AS bucket, approx_percentile(latency_ms, .95)` | Feeds org-level latency chart. |

### Deferred backlog (outside current scope)

- **Incident drilldowns** (failure streak boundaries, geo impact, recovery latency charts).
- **Cost & pipeline health** (AE write volume, dispatch runtime percentiles, write error alerts).

Keep any prior notes for these flows in git history; revisit once monitor-health dashboards ship.

## Validation Checklist

1. **Schema audit** – Confirm `apps/backend/src/internal/dispatch.rs` emits every field listed in the heartbeat fact row.
2. **Backfill strategy** – Document whether we need to seed AE from D1 when adding new columns (especially `dispatch_id`, `sample_rate`).
3. **API coverage (monitor health only)** – Ensure `GET /api/monitors/:id/heartbeats`, `/uptime`, `/latency`, and fleet-level endpoints have specs and owners.
4. **Sampling transparency** – Every AE-backed response must echo the effective sample rate so UI can show fidelity badges.

## Next Steps

1. Review this requirements list with design/PM to confirm the monitor-health scope is complete.
2. Update the AE schema + ingestion worker to include any missing columns (especially `dispatch_id`, `region`, `sample_rate`).
3. Implement the monitor-detail and fleet-level backend queries/APIs, then wire the corresponding frontend surfaces.
4. Add synthetic tests/alerts that validate AE freshness and field coverage as part of CI or canary environments.

This doc stays focused on functional expectations; implementation details belong in `analytics-engine-plan.md` and module-specific READMEs.
