## CF-Native Uptime Monitor (Saavy Uptime)

A Cloudflare-first uptime and incident platform: Rust Workers + Durable Objects for sub-minute scheduling, D1 for hot configuration/heartbeats, Analytics Engine for aggregates, R2 for archival, and Cloudflare Access-secured frontend hosted on Pages.

### Repository Layout

- `apps/frontend/` – Vite/React dashboard + status pages.
- `apps/worker/` (future) – Rust worker + Durable Object scheduler.
- `docs/` – Architectural references, including `the-plan.md` and `implementation-plan.md`.

### Implementation Roadmap

The detailed phased roadmap, shipping timeline, and getting-started checklist live in `docs/implementation-plan.md`. Highlights:

1. **Foundations:** Wrangler config, Access policies, D1 schema, CI.
2. **Monitor CRUD + Scheduler:** API/UI + Durable Object ticker that claims jobs.
3. **HTTP Checks & Storage:** Real checks, D1 heartbeats (24 h retention), AE metrics, incidents.
4. **Dashboard & Status Pages:** Internal dashboard + public `/status/<slug>`.
5. **Notifications:** Webhook/email alerts with retries and delivery logs.
6. **Data Lifecycle & Polish:** R2 archival cron, import/export, observability, automation tokens.

### Getting Started

1. Install Wrangler, Rust, Node, PNPM (or npm), **Task** (`go install github.com/go-task/task/v3/cmd/task@latest`), and `cloudflared` locally.
2. Configure Cloudflare account/environment variables for D1, AE, R2, and Access. Set `ACCESS_TEAM_DOMAIN` (e.g., `your-team.cloudflareaccess.com`) and the Access application audience string `ACCESS_AUD` in `wrangler.toml` (and per-environment overrides if needed).
3. Run initial D1 migrations and seed scripts (`wrangler d1 migrations apply <db>`).
4. Start the worker/dev server (`wrangler dev`) and frontend (`pnpm dev`) to iterate.

`cloudflared` lets you run Access-authenticated dev traffic.  Use `cloudflared access login <https://your-app>` followed by `export VITE_CF_ACCESS_TOKEN=$(cloudflared access token --app https://your-app)` (or set it in `.env.local`) before `pnpm dev`. That injects the `CF_Authorization` header into Vite-powered requests until we have a first-class proxy.

For deeper context on functional requirements and architecture see `docs/the-plan.md`; pair it with the roadmap doc to track progress. We’re also collecting WASM/Workers gotchas in `docs/workers-rs-notes.md`.

### Deployment Steps (Phase 0)

1. **Authenticate Wrangler:** `wrangler login` with the Cloudflare account that owns the target zone.
2. **Provision data stores:**
   - D1: `wrangler d1 create saavy_uptime_dev` (repeat for preview/prod). Copy the generated `database_id` into the matching `wrangler.toml` blocks.
   - Analytics Engine: `wrangler analytics-engine create saavy_uptime_dev` (and preview/prod variants).
   - R2: `wrangler r2 bucket create saavy-uptime-dev` (and preview/prod variants).
3. **Durable Object namespace:** `wrangler deploy --dry-run` (or `wrangler do create TICKER --class Ticker`) to ensure the `TICKER` Durable Object namespace exists before real deploys.
4. **Backend build + deploy:** use Taskfile helpers.
   - `task backend:check`
   - `task backend:build`
   - `wrangler deploy --env preview` (use `--env production` once resources are wired to prod IDs).
5. **Frontend build + Pages deploy:**
   - `task frontend:build`
   - `wrangler pages deploy dist --project-name=saavy-uptime`
   - Configure the custom domain via the Cloudflare Pages dashboard or `wrangler pages project domain add`.
6. **Secrets & Access:** No Wrangler secrets are required yet; when APIs introduce sensitive configuration, set them with `wrangler secret put <NAME>` per environment and document them here.

### Infrastructure as Code

We are standardizing on Cloudflare's Terraform provider to provision Worker routes, D1/AE/R2 resources, DNS records, and Access applications/policies. The current deploy steps still assume manual setup, but future iterations will ship Terraform modules (plus helper scripts) so a new team can `terraform apply` and immediately have Workers, Access, DNS, and onboarding-ready infrastructure. Contributions to the Terraform scaffolding are welcome; track progress in `docs/implementation-plan.md`.

**Manual steps to codify**

- DNS records for the custom domains (e.g., CNAME/AAAA for Workers + Pages).
- Cloudflare Zero Trust self-hosted application for the Worker/Pages hostname.
- Access policy that allows the right emails/domains/groups (may stay partially manual if per-team policies differ).
