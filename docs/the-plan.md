### **Project: CF-Native Uptime Monitor**

#### Why

Most self-hosted uptime tools assume persistent disks and heavyweight runtimes. Cloudflare’s global edge environment offers the inverse: ephemeral compute, edge-storage primitives, advanced stateful scheduling, and built-in auth.

This project re-imagines an uptime platform that:

  * deploys with zero servers or volumes,
  * stores configuration and **"hot" data in D1**,
  * archives **"cold" data to R2**,
  * handles high-volume time-series aggregates in **Analytics Engine**,
  * runs stateful, sub-minute checks using **Durable Objects**, and
  * secures admin endpoints with **Cloudflare Access**.

**Goal:** A fast, globally distributed, cheap-to-run, and scalable uptime dashboard that feels like a native Cloudflare product.

### Functional Requirements

#### Core

  * **HTTP/HTTPS Monitors:** Configurable interval (**15 s – 60 min**), timeout, expected status/sub-string, redirect & TLS verification flags.
  * **Tags, enable/disable, simple search.**

#### Check Execution

  * **Stateful Sub-Minute Scheduling:** A central **Durable Object (DO) "ticker"** uses its `alarm()` API to create a reliable, persistent 15-second loop.
  * **Dispatcher Logic:** The DO queries D1 for monitors due to be run and dispatches checks as stateless Worker invocations.
  * **At-least-once delivery;** automatic retry & backoff.

#### Results Storage

  * **Hot Storage (D1):** Store per-check raw results (pass/fail, RTT, error) in **D1**. This data is treated as a **30-day rolling log** for immediate incident forensics.
  * **Aggregate Metrics Storage (AE):** Write key metrics (e.g., `rtt_ms`, `ok_status`) to **Cloudflare Analytics Engine (AE)**. This is the high-performance store for all dashboard aggregates.
  * **Cold Storage (R2):** A daily "janitor" Worker (on a Cron Trigger) will find heartbeats older than 30 days, **archive them to R2** (e.g., as gzipped JSON or Parquet files), and then **prune** them from D1.
  * **Data Lifecycle:** This strategy keeps the D1 database fast and well within size limits, while R2 provides a cheap, long-term archive for historical data.

#### Incident Logic

  * Open after N consecutive failures, close after M successes.
  * Maintain incident timeline + summary stats in D1.

#### Notifications

  * Webhooks + Email via MailChannels/Resend; per-monitor policy.
  * Signed payloads; retry on 5xx.

#### Status Pages

  * Public HTML + JSON endpoint per org.
  * Dashboard queries **Analytics Engine** for fast uptime % (24 h/7 d/30 d) and latency graphs (p50/p90/p99).
  * Current state and recent incidents queried from D1.
  * Light/dark themes, custom slug.

#### Import/Export

  * JSON import of monitors; JSON export of configuration + incidents.
  * Future support for exporting historical (R2-archived) data.

#### Administration

  * Admin UI protected by Cloudflare Access.
  * Bypass rules for `/status/*` and public JSON.

#### Automation Access

  * Optional Access Service Tokens for CI/CLI; no local auth.

#### Nice-to-Have Later

  * **ICMP probes** via container sidecar (Ping is not possible from Workers).
  * **Browser/screenshot checks** using puppeteer + R2 blobs.
  * Maintenance windows & SLO reports.
  * Multi-user orgs + roles (possibly via D1 database-per-tenant sharding).
  * Expanded notifications (Slack, Pushover, Twilio).

### Non-Functional Requirements

| Category | Target |
| :--- | :--- |
| **Performance** | p95 API \< 150 ms; dashboard \< 1.5 s @ 50 monitors (queries AE) |
| **Cost** | \<$5/mo on CF Paid Plan (Workers, DO, D1, AE, R2 usage) |
| **Reliability** | Idempotent writes, retries, at-least-once guarantee via DO alarms |
| **Scalability** | Horizontally elastic via Workers; DO for state; D1/R2 for data lifecycle |
| **Security** | HSTS, CSRF guards, signed webhooks, CF Access Auth |
| **Maintainability** | All logic in Workers/DO; zero VMs; one-click deploy |
| **Observability** | Structured logs + metrics in Analytics Engine |
| **Availability** | 99.9 % target (dependent on CF platform) |

### Proposed Data Schema (D1)

```sql
-- Users/Orgs (optional future expansion)
create table organizations (
  id text primary key,
  slug text unique,
  created_at integer
);

-- Monitors
create table monitors (
  id text primary key,
  org_id text not null,
  name text not null,
  kind text not null default 'http', -- 'http', 'tcp', 'dns'
  url text not null,
  interval_s integer not null,
  timeout_ms integer not null,
  follow_redirects integer not null,
  verify_tls integer not null,
  expect_status_low integer,
  expect_status_high integer,
  expect_substring text,
  headers_json text,
  tags_json text,
  enabled integer not null,
  last_checked_at_ts integer, -- Used by the DO scheduler
  created_at integer not null,
  foreign key(org_id) references organizations(id)
);

-- Heartbeats (raw results, 30-day "hot" store for forensics)
create table heartbeats (
  monitor_id text not null,
  ts integer not null,
  ok integer not null,
  code integer,
  rtt_ms integer,
  err text,
  primary key (monitor_id, ts)
);

-- Note: The 'rollup_minute' table is no longer needed.
-- All aggregates (p50/p90/p99, avg, count) will be
-- handled by the Cloudflare Analytics Engine.

-- Incidents
create table incidents (
  id text primary key,
  monitor_id text not null,
  opened_ts integer not null,
  closed_ts integer,
  reason text,
  foreign key(monitor_id) references monitors(id)
);

-- Notifications
create table notifications (
  id text primary key,
  monitor_id text not null,
  kind text not null,
  target text not null,
  created_at integer not null
);

-- Settings
create table settings (
  key text primary key,
  val text not null
);
```

### Architecture Overview

#### Runtime Stack

  * **Frontend:** Vite + React → Cloudflare Pages (static)
  * **Backend:** Rust Worker (WASM) for the API
  * **Storage (Relational / Hot):** **D1** for configuration, incidents, and **30 days of raw heartbeats**.
  * **Storage (Analytics):** **Analytics Engine** for all time-series aggregates (latency, uptime).
  * **Storage (Blobs / Cold):** **R2** for **archived raw logs** (older than 30 days) and future blobs (screenshots, exports).
  * **Scheduling:** **Durable Objects** for stateful, sub-minute scheduling (`alarm()` API)
  * **Auth:** Cloudflare Access (no local auth code)
  * **Notifications:** MailChannels / Resend / generic webhook
  * **Public Endpoints:** `/status/*` bypasses Access; cached globally

#### Why This Approach Works

  * **Cloudflare-native:** Every component (compute, state, analytics, storage, auth) lives within the same platform.
  * **Easy Deploy:** `wrangler deploy` is the whole install; no volumes or databases to provision.
  * **Secure by Default:** Auth delegated to CF Access with SSO and audit logging included.
  * **Stateful at the Edge:** Durable Objects solve the sub-minute scheduling problem reliably.
  * **Performant Analytics:** Leverages Analytics Engine for near-instant percentile (p50/p90/p99) and aggregate queries.
  * **Manageable Data Scale:** The D1(hot) -\> R2(cold) data lifecycle prevents unbounded database growth, keeping D1 fast and cheap while preserving all historical data.

### MVP Scope Summary

| Area | Deliver in v1 | Defer |
| :--- | :---: | :--- |
| **HTTP/HTTPS Checks** | ✅ | — |
| **Dashboard & Status Page** | ✅ | — |
**D1/R2 Data Pruning** | ✅ | — |
| **Notifications (Webhook + Email)** | ✅ | — |
| **CF Access Auth** | ✅ | — |
| **TCP/DNS Checks** | — | ⏩ v2 (via `connect()` API & DoH) |
| **Browser Checks / Screenshots** | — | ⏩ Later |
| **ICMP (Ping) Checks** | — | ⏩ Later (requires sidecar) |
| **Multi-User Orgs + Roles** | — | ⏩ Later |
| **Maintenance Windows / SLOs** | — | ⏩ Later |
| **R2 Log *Retrieval* (UI)** | — | ⏩ Later (Archive *to* R2 is v1) |

**Summary:**
A clean-slate, Cloudflare-native uptime monitor: React UI, Rust edge worker, **Durable Object** scheduler, **D1** for hot data, **AE** for metrics, **R2** for cold data, and **CF Access** for auth. Zero servers, zero volumes, one-command deploy.