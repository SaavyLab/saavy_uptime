# Analytics Engine

How we use Cloudflare Analytics Engine (AE) alongside D1 for heartbeat storage, historical aggregates, and dashboard queries.

## Goals

- **Historical aggregates** - Surface uptime %, latency percentiles, regional coverage over 24h / 7d / 30d without hitting D1 for every request.
- **Configurable cost knob** - Let hobbyists dial AE usage down via sampling, while larger teams can push more data for richer analytics.
- **No critical loss** - AE sampling must never make the product lie. D1 holds the source of truth for current monitor state and incidents.

## Data Flow

1. **Dispatch runner** executes a monitor check (HTTP/TCP).
2. **D1 update** - Hot state (current status, `last_checked_at`, incident state) written to D1.
3. **AE write** - If `rand() < sample_rate`, heartbeat summary written directly to AE via `AnalyticsEngineDataset.write_data_point()`.

D1 no longer keeps per-heartbeat logs; it only tracks latest monitor status/incident state. AE is the source for heartbeat history.

## Schema

Heartbeats are written via `write_heartbeat_to_analytics()` in `apps/backend/src/internal/dispatch.rs`.

| AE Column | Field | Type | Purpose |
| --- | --- | --- | --- |
| `index1` | `monitor_id` | string | Primary index for per-monitor queries |
| `blob1` | `org_id` | string | Authorization + org-level aggregates |
| `blob2` | `dispatch_id` | string | Incident correlation, dedup |
| `blob3` | `status` | string | `"up"` / `"down"` / `"pending"` |
| `blob4` | `region` | string | Logical region from `cf.region()` |
| `blob5` | `colo` | string | Cloudflare POP code from `cf.colo()` |
| `blob6` | `error` | string | Error message (empty if success) |
| `double1` | `timestamp_ms` | f64 | Unix timestamp in milliseconds |
| `double2` | `latency_ms` | f64 | Round-trip time |
| `double3` | `code` | f64 | HTTP status code (0 if N/A) |
| `double4` | `sample_rate` | f64 | For unbiased estimates when sampling < 1.0 |

## Query Examples

Queries are executed via `AeQueryClient` in `apps/backend/src/analytics/client.rs`. See `monitor_health.rs` for the primary query.

### Recent Heartbeats (Monitor Detail)

```sql
SELECT
    index1 as monitor_id,
    blob1 as org_id,
    blob2 as dispatch_id,
    double1 as timestamp_ms,
    blob3 as status,
    blob4 as region,
    blob5 as colo,
    blob6 as error,
    double2 as latency_ms,
    double3 as code,
    double4 as sample_rate
FROM {dataset}
WHERE index1 = '{monitor_id}'
  AND blob1 = '{org_id}'
  AND double1 BETWEEN {since_ms} AND {until_ms}
ORDER BY double1 DESC
LIMIT 100
FORMAT JSON
```

### Uptime Percentage (planned)

```sql
SELECT
    SUM(CASE WHEN blob3 = 'up' THEN double4 ELSE 0 END) / SUM(double4) as uptime_pct
FROM {dataset}
WHERE index1 = '{monitor_id}'
  AND double1 >= {since_ms}
FORMAT JSON
```

Note: `double4` is `sample_rate` used as weight for unbiased estimates.

## API Endpoints

| Endpoint | Description | Source |
| --- | --- | --- |
| `GET /api/monitors/:id/heartbeats` | Recent heartbeats for a monitor | `monitor_health::recent_heartbeats()` |
| `GET /api/monitors/:id/uptime` | Rolling uptime % (planned) | - |
| `GET /api/monitors/:id/latency` | Latency percentiles (planned) | - |

## Configuration Knobs

- `AE_SAMPLE_RATE` (per org, default 1.0) - Lowering reduces write volume but increases variance. Rates < 0.5 degrade uptime accuracy.
- `AE_HEARTBEATS_DATASET` - Dataset name configured in `wrangler.toml`.
- `AE_ACCOUNT_ID` - Cloudflare account ID for API queries.
- `AE_API_TOKEN` - Secret store token for read queries.

When sampling is enabled:
1. Rate persisted in D1 org settings.
2. `should_record(sample_rate)` gates writes in the dispatch path.
3. `sample_rate` stored with each row so queries can use it as weight.
4. Dashboard shows fidelity banner when running in low-sample mode.

## What Stays in D1 (Not AE)

- Monitor configuration (URL, interval, thresholds)
- Current hot state (`status`, `last_checked_at`, `consecutive_failures`)
- Incident records and timeline
- Org/user settings

AE queries reference D1 when joining monitor metadata but are not the source of truth for state.
