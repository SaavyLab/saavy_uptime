# Roadmap – CF-Native Uptime Monitor

## Overview

This roadmap breaks the project into incremental, shippable phases so we can deploy value quickly while keeping architecture aligned with Cloudflare-native goals. Each phase includes deliverables, key tasks, and enabling milestones.

## Phase 0 – Foundations

- **Deliverable:** Skeleton deployment with Wrangler-configured Worker, DO, D1, AE bindings, Cloudflare Access, and React shell.
- **Tasks:**
  - Establish repository structure (Workers backend, Durable Object module, frontend app).
  - Configure `wrangler.toml`, environments, bindings, and Access policies.
  - Define initial D1 schema + migrations, plus seed script for local dev.
  - Set up CI (lint, type-check, unit tests) and deploy workflow.

## Phase 1 – Monitor CRUD & Scheduler Skeleton

- **Deliverable:** Users can manage monitors; Durable Object ticker schedules runs but only logs dispatches.
- **Tasks:**
  - Build D1-backed CRUD API for monitors, notification policies, and org settings.
  - Implement React UI for create/edit/list with search, tags, enable/disable toggles.
  - Design scheduler data model (`next_run_at`, lease tokens) and implement DO `alarm()` loop that claims work.
  - Add worker endpoint to receive dispatch requests (no real checks yet) and emit structured logs.

## Phase 2 – HTTP Check Execution & Analytics Engine Writes

- **Deliverable:** Monitors execute real HTTP/HTTPS checks; D1 maintains monitor “hot state” while Analytics Engine stores heartbeat summaries; incidents open/close.
- **Tasks:**
  - Implement fetch-based checker with timeout, redirects, TLS verification, status/sub-string assertions.
  - Update the dispatch path to write heartbeats directly to AE (respecting per-org sampling) after updating D1 hot state.
  - Add incident state machine (N failures to open, M successes to close) and store incident timeline.
  - Surface incident summaries in API and basic UI widgets.

## Phase 3 – Dashboard & Public Status Pages

- **Deliverable:** Internal dashboard and per-org public status page backed by AE + D1.
- **Tasks:**
  - Build dashboard views querying AE for uptime/latency aggregates and D1 for incidents.
  - Create public `/status/<slug>` HTML + JSON endpoints bypassing Access but cached via Workers.
  - Implement light/dark theme toggle and simple customization settings.
  - Add charts (p50/p90/p99 latency, uptime cards for 24 h/7 d/30 d) leveraging AE queries.

## Phase 4 – Notifications

- **Deliverable:** Monitor-specific policies send webhook + email alerts with retries and audit logs.
- **Tasks:**
  - Integrate MailChannels/Resend + generic webhook channel with signed payloads.
  - Create `notification_events` table storing incident ID, attempt, status, and retry metadata.
  - Implement retry/backoff on 5xx responses and expose delivery log UI.
  - Allow per-monitor notification configuration in frontend.

## Phase 5 – Polish & Stretch

- **Deliverable:** Access service tokens, JSON import/export flows, maintenance window scaffolding, observability enhancements.
- **Tasks:**
  - Issue Access service tokens for automation; document usage.
  - Build JSON import/export for monitors/incidents and optional artifact attachment hooks (future R2 usage).
  - Add maintenance window + SLO placeholder models for future growth.
  - Instrument structured logs, tracing, AE dashboards, and cost/perf tests.

## Shipping Timeline (Example)

1. **Week 1 – Bootstrap:** Repo scaffolding, Wrangler deploy, Access, CI, migrations.
2. **Week 2 – CRUD + Scheduler:** API/UI for monitors, DO ticker logging dispatches.
3. **Week 3 – Execution + AE Writes:** Real HTTP checks, AE metrics, incident engine.
4. **Week 4 – Dashboard & Status:** Internal dashboard, public status pages, AE charts.
5. **Week 5 – Notifications:** Email/webhook delivery with retries and log UI.
6. **Week 6 – Hardening:** Load/cost tests, docs, Access tokens, maintenance mode stubs.

## Getting Started Checklist

- Create Wrangler project with bindings for D1 dev/prod databases, AE datasets, and Durable Object namespace.
- Write initial SQL migrations and integrate `wrangler d1 migrations apply` into CI/deploy scripts.
- Scaffold frontend via Vite/React hosted on Cloudflare Pages, secured through Access.
- Build monitor CRUD API endpoints plus seed data to simulate monitors locally.
- Implement Durable Object scheduler with `alarm()` loop, `next_run_at` calculations, and logging for dispatched job IDs.

Once the checklist is complete, begin iterating on Phase 1 tasks and use feature flags/env toggles to keep incomplete features dark while still deploying frequently.
