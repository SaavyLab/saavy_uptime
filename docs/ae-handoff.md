# AE Telemetry & Insights – Handoff Notes

## Current State
- Dispatch pipeline, ticker DO batching, typed configs, and frontend monitor list enhancements are merged; workers now emit structured heartbeat payloads and D1 stores monitor snapshots.
- Analytics Engine (AE) is not yet wired up; all insights, charts, and heartbeat history views still read from D1 or in-memory data.
- Highlight features (execution DAG, geo map, predictive cost) depend on AE-backed timeseries queries and aren’t implemented yet.

## Objectives for Next Chunk
1. **Validate AE schema coverage**
   - Ingestion via `internal::dispatch::persist_heartbeat_result` already writes heartbeat summaries to AE; confirm the stored columns cover upcoming analytics and extend the schema if we need extra dimensions such as dispatch IDs.
   - Add instrumentation/alerts so the synchronous AE writes stay healthy (<1 min lag); log/alert on failures since there’s no queue retry.

2. **Query APIs for insights**
   - Add backend endpoints (e.g., `/api/monitors/:id/heartbeats`, `/api/insights/fleet`) that run AE SQL via the Analytics Engine client.
   - Support basic aggregations first: recent heartbeats (limit 100), uptime % over time window, p95 latency per monitor/region.

3. **Frontend surfaces**
   - Monitors detail page: sparkline or table showing recent heartbeats, uptime badge sourced from AE query, latency chart.
   - Dashboard cards: org-level uptime, average latency, failure count in last hour.

## Suggested Work Breakdown
1. **Schema + ingestion**
   - Author AE DDL + wrangler binding updates.
   - Implement ingestion path (synchronous write in dispatch vs. async worker) and add retries/metrics.

2. **Backend query helpers**
   - Create `analytics` module with reusable AE client + typed response structs.
   - Implement functions like `fetch_recent_heartbeats(monitor_id, limit)` and `fetch_org_latency(org_id, window)`.

3. **API & UI**
   - Expose AE data via new REST endpoints.
   - Update frontend hooks/components to consume those endpoints, with loading/error states.

## Open Questions / Decisions
- Retention + aggregation: do we need downsampling/rollups for long windows, or is raw heartbeat storage sufficient for now?
- Authentication/authorization for AE queries—mirror existing org scoping from D1 service layer.

## Dependencies & References
- `docs/highlight-features.md` for the long-term visualization ideas.
- `apps/backend/src/internal/dispatch.rs` for heartbeat generation.
- Cloudflare AE docs for query syntax + API limits.

## Definition of Done for First Iteration
- Heartbeats reliably stored in AE with <1 min lag.
- API endpoint delivering recent heartbeats + uptime stats for a monitor.
- Frontend section rendering that data (table/chart) without regressions in existing monitor list.
