# Analytics Engine Plan

How we’ll use Cloudflare Analytics Engine (AE) alongside D1/R2, what we log, and how we keep costs predictable without sacrificing accuracy.

## Goals

- **Historical aggregates** – Surface uptime %, latency percentiles, regional coverage, and cost projections over 24 h → 30 d without hitting D1 for every request.
- **Configurable cost knob** – Let hobbyists dial AE usage down, while larger teams can push more data for richer analytics.
- **No critical loss** – AE sampling must never make the product lie. Detail views stay backed by D1 so even low sampling doesn’t hide real incidents.

## Data we stream to AE

| Category | Fields | Critical? | Used for | Sampling strategy |
| --- | --- | --- | --- | --- |
| **Heartbeat summary** | `monitor_id`, `org_id`, `ts`, `ok`, `code`, `rtt_ms`, `region` | **Yes** | Uptime %, latency percentiles, POP coverage | Default 1.0 (every heartbeat). Sample rate is configurable but we warn users when precision drops. |
| **Incident context** | `consecutive_failures`, `current_incident_id`, `previous_status` | Optional | MTTR, incident timelines, detection latency charts | Same sample rate as heartbeats; roll up per monitor. |
| **Cost telemetry** | `dispatch_duration_ms`, `d1_reads`, `d1_writes`, `ae_writes` per batch | Optional | Predictive cost dashboard, “turn the knobs” UI | Sample once per dispatch batch (or per minute) to keep AE write volume flat. |
| **Notification events** | `monitor_id`, `notification_kind`, `status`, `attempt_ts` | Optional (phase 4+) | Delivery reliability charts | Usually sparse; can log all events without sampling. |

Notes:

- If sampling < 1.0, we still need unbiased estimates. We can store the sample rate with each heartbeat row (e.g., `sample_weight = 1 / rate`) so AE queries can `SUM(ok * sample_weight)` and produce correct percentages.
- Even when sampling is off, D1 retains every heartbeat for ~24 h, and R2 handles archival. AE is purely for aggregated reads.

## What stays out of AE

- **Hot heartbeats / detail history** – The monitor detail page and recent timelines still read from D1. Users never lose immediate visibility when they lower AE sampling.
- **Configuration, incidents, notifications** – authoritative data stays in D1 tables. AE can mirror summaries but not the source of truth.
- **R2 archives** – long-term raw logs still ship to R2 for auditors/debugging.

## Reads & UI usage

- **Dashboard cards** – Uptime (24 h/7 d/30 d), latency percentiles, POP coverage read from AE. If sample rate is too low, show a message (“Analytics disabled: AE sampling < 0.5”) and fall back to coarse estimates.
- **Status pages** – Public uptime history uses AE aggregates to avoid hammering D1.
- **Cost console** – AE stores resource usage so we can plot projected CF bill vs. monitor count.
- **Detail view** – stays on D1; AE isn’t involved.

## Configuration knobs

- `AE_SAMPLE_RATE` (per org) – 1.0 by default. Lowering it reduces write volume but increases variance. We warn users that sample rates < 0.5 degrade uptime accuracy.
- `AE_HEARTBEAT_BATCH_SIZE` – optional throttle (write summary once per N heartbeats).
- `AE_RETENTION_DAYS` – partitions/TTL for AE tables; defaults to 90 days.

Whenever the user lowers the sample rate, we:

1. Persist the new rate in D1 (org settings).
2. Apply it to the Worker pipeline (e.g., only write heartbeat summaries when `rand() < rate`).
3. Store `sample_rate` or `sample_weight` column with each AE row so queries can compensate.
4. Surface a banner in the dashboard if analytics are running in “low-fidelity” mode.

## Next steps

1. Design the AE table schema (likely one dataset per environment with columns above).
2. Add a queue producer in the dispatch path that enqueues heartbeat summaries (respecting rate).
3. Implement a `heartbeat-summaries` queue consumer Worker that batches messages and writes to AE (one dataset per environment).
4. Build the first AE query (uptime last 24 h) and wire it into the dashboard.
5. Add UI copy/docs explaining the cost vs. fidelity trade-off when adjusting AE settings.

## Queue-based ingestion

- **Producer:** The dispatch runner (internal Worker route) publishes each heartbeat summary to the `HEARTBEAT_SUMMARIES` queue. Messages include `sample_rate` metadata so AE queries can correct for sampling.
- **Consumer:** A separate queue consumer Worker (defined under `apps/backend/src/external/queues/heartbeat_summaries.rs`) receives batches (e.g., 100 messages), groups them by dataset, and writes to AE via the Analytics Engine API. Any AE write failure can be retried because the queue only `ack`s after a successful flush.
- **Config:** Wrangler config (`wrangler.toml`) declares the queue producer/consumer bindings with conservative `max_batch_size`/`max_batch_timeout`. Inside the consumer Worker, we honor org-specific batch limits via env vars if needed.

This approach keeps the dispatch path lean (fire-and-forget queue publish) while giving us durable batching/retries for AE writes.
