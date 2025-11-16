# Saavy Uptime

Cloudflare-native uptime and incident monitoring: Rust Workers + Durable Objects for sub‑minute scheduling, D1 for hot configuration/heartbeats, Analytics Engine for aggregates, R2 for archival, and a Vite/React dashboard served via Cloudflare Pages. Everything in this repo is open source and deploys into your own Cloudflare account so you control the knobs *and* the bill. Running entirely on Cloudflare’s global edge means no servers, no cron VMs, no regional outages, and sub-50ms cold starts for every check.

> **Status:** Actively in development. Phase 1/2 of the implementation plan (monitor CRUD + scheduler + HTTP checks) is underway—expect APIs and UI to evolve quickly.

---

## Why Saavy Uptime?

Saavy Uptime is for builders who already trust Cloudflare:

- **Hobbyists & small/medium teams** using Workers/Pages who want matching uptime coverage without renting new servers or paying heavyweight SaaS plans.
- **Hybrid Cloudflare users** who rely on Cloudflare for DNS/CDN but host workloads on ECS/Kubernetes and don’t want to run a parallel monitoring fleet.
- **Game/server admins** looking for high-frequency, low-cost checks with full control over intervals, retention, and analytics.
- **Cloudflare-first teams who want monitoring that runs on the same global edge as their workloads.

Instead of buying uptime from someone else, you deploy this Worker + Durable Object stack into your Cloudflare account. Run 30 monitors at 60 s for pennies, dial up to hundreds at 15 s for roughly $100–$200/month, or park everything somewhere in between—the only invoice you see is from Cloudflare.

---

## Highlights

- **Sub-minute scheduling** – Durable Object alarms claim due monitors in batches and dispatch immediately, no cron drift or VM orchestration.
- **Cloudflare-first architecture** – Axum Worker secured by Access, DO ticker for stateful scheduling, D1 for config + heartbeats, Analytics Engine for aggregates, R2 for cold storage.
- **Bring-your-own cost model** – tweak intervals, retention, and aggregation to match your budget; you pay Cloudflare directly.
- **One-click deploy** – Wrangler + Pages handle everything; no extra infrastructure to babysit.
- **Stretch goals** – real-time execution DAG, geographic POP map, predictive cost dashboard, R2 time-travel incident replay, collaborative incident timelines (`docs/highlight-features.md`).

---

## Repository Layout

- `apps/backend/` – Rust Worker (Axum) plus Durable Object integrations.
   This is the control plane, scheduling layer, and dispatch runner
- `apps/frontend/` – Vite/React dashboard, status shell, routing.
   All internal dashboard + public status page UI.
- `docs/` – Implementation plan, design notes, Cloudflare differentiator ideas.
   These explain how DO alarms, AE, D1, and R2 fit together

---

## Getting Started (local dev)

Prerequisites:

- Rust stable + `wasm32-unknown-unknown`
- `cargo install worker-build`
- Wrangler v3
- Node 20+, PNPM
- `cloudflared` (for Access-authenticated local requests)
- Task (`go install github.com/go-task/task/v3/cmd/task@latest`)

### Install & run

```bash
task backend:install
task frontend:install
```

1. **Configure Access/D1 bindings** – set `ACCESS_TEAM_DOMAIN`, `ACCESS_AUD`, and D1 IDs in `wrangler.toml`. For the frontend, export `VITE_CF_ACCESS_TOKEN` (via `cloudflared access login https://<your-app>`).
2. **Apply migrations**
   ```bash
   wrangler d1 migrations apply <db-name>
   ```
3. **Start dev servers**
   ```bash
   task backend:dev    # wrangler dev
   task frontend:dev   # pnpm dev
   ```

Need demo data? During development you can seed ~400 monitors hitting httpbin/httpstat/postman endpoints:

```bash
curl -X POST http://localhost:8787/api/internal/seed \
  -H "Content-Type: application/json" \
  -H "Cf-Access-Jwt-Assertion: $VITE_CF_ACCESS_TOKEN"
```

---

## Roadmap Snapshot

1. **Foundations** – Wrangler config, Access, D1 schema, CI (done).
2. **Monitor CRUD + ticker** – API/UI + Durable Object scheduler (in progress).
3. **HTTP execution & storage** – Real checks, D1 heartbeats, AE metrics, incident skeleton.
4. **Dashboard & status pages** – AE-backed aggregates, public `/status/<slug>`.
5. **Notifications** – Webhook/email policies, retries, delivery logs.
6. **Lifecycle & polish** – R2 archival cron, import/export, service tokens, “wow” visualizations.

Full phase details live in `docs/implementation-plan.md`.

---

## Inspiration

- [Uptime Kuma](https://github.com/louislam/uptime-kuma) for its approachable UX and DIY ethos.
- Cloudflare’s platform itself—Workers, Durable Objects, Access, D1, Analytics Engine, R2—showing what a zero-server monitoring stack can look like.
- Indie SaaS teams, hobby projects, and game communities that want the power of Cloudflare’s edge without the cost or complexity of running separate monitoring infrastructure.

---

## Contributing

100 % open source from day one. If you are experimenting with Workers/Durable Objects, chasing Cloudflare-native observability, or just want to help, contributions are welcome:

- File issues/ideas, especially around scheduler design, heartbeats, AE queries, or UX polish.
- Share your own cost presets or deployment guides so other teams can replicate cheap setups.
- Help with documentation, Terraform automation, or the differentiator features listed above.

---

## Stay in the loop

- Watch the repo for release notes as the MVP stabilizes.
- Check the `docs/` folder for implementation updates and Cloudflare-specific lessons.
- Cloudflare folks: reach out via Issues/Discussions if you want to collaborate—we’d love feedback on pushing the platform’s limits.

---

Built with ❤️ for the Cloudflare community. Let’s make uptime monitoring as effortless as deploying a Worker.
