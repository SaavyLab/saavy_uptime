# Deployment Guide

Living runbook for getting Saavy Uptime into any Cloudflare environment today while we work toward a true one-click experience driven by `wrangler.toml`, Taskfile helpers, and future Terraform modules.

## 1. Tooling & Access

| Requirement | Notes |
| --- | --- |
| Cloudflare account + target zone | Need access to create Workers, Durable Objects, D1, AE, R2, Pages, and Access apps. |
| CLI utilities | `wrangler` ≥ v3, Rust stable + `wasm32-unknown-unknown`, `cargo install worker-build`, Node 20+/Bun, PNPM (or npm), `go-task`, `cloudflared`. |
| Auth | `wrangler login`, `cloudflared access login https://<app>` then export `VITE_CF_ACCESS_TOKEN` for local/preview smoke tests. |
| Secrets/config | `ACCESS_TEAM_DOMAIN`, `ACCESS_AUD`, `DISPATCH_TOKEN`, `DISPATCH_BASE_URL`, etc. live in `wrangler.toml` (per-env overrides encouraged). |

## 2. Environment Matrix

| Env | Purpose | Cloudflare bindings |
| --- | --- | --- |
| `dev` (default) | local + tunnel experiments | `DB=saavy_uptime_dev`, `AE=saavy_uptime_dev`, `ARCHIVE_BUCKET=saavy-uptime-dev`, `TICKER` DO namespace (migration `v1`). |
| `preview` | shared staging / PR previews | Override bindings under `[env.preview]`, enable `workers_dev = true` for quick links. |
| `production` | customer-facing | Dedicated resources under `[env.production]`, `workers_dev = false`, Pages custom domains. |

Set the `database_id`, AE dataset, and R2 bucket identifiers per environment inside `wrangler.toml` before deploying.

## 3. Provision Cloudflare Resources (manual for now)

1. **Workers + Durable Object namespace**
   ```bash
   wrangler deploy --dry-run            # creates DO migrations
   wrangler do create TICKER --class Ticker
   ```
2. **D1 databases** (dev/preview/prod):
   ```bash
   wrangler d1 create saavy_uptime_preview
   wrangler d1 migrations apply saavy_uptime_preview
   ```
3. **Analytics Engine datasets**:
   ```bash
   wrangler analytics-engine create saavy_uptime_preview
   ```
4. **R2 buckets**:
   ```bash
   wrangler r2 bucket create saavy-uptime-preview
   ```
5. **Access application + policy**: create a self-hosted app covering the Worker + Pages hostname; capture the team domain and audience string for `wrangler.toml`.
6. **Pages project** (one-time):
   ```bash
   wrangler pages project create saavy-uptime
   ```

> Terraform backlog: each bullet above maps cleanly to `cloudflare_workers_script`, `cloudflare_d1_database`, `cloudflare_analytics_engine_dataset`, `cloudflare_r2_bucket`, `cloudflare_access_application`, and `cloudflare_pages_project`. Documenting these manual steps now lets us codify them later without guesswork.

## 4. Repo Configuration

1. Copy `.env.example` (when available) or export `ACCESS_*`, `VITE_*`, and `DATABASE_NAME` locally.
2. Update `wrangler.toml` with the resource IDs you just created:
   - default section for dev,
   - `[env.preview]` overrides,
   - `[env.production]` overrides.
3. Double-check `Taskfile.yaml` inherits the right project/bucket names (e.g., `PROJECT_NAME`, backend `DATABASE_NAME` var).

## 5. Build & Validate Locally

```bash
task install
task backend:check
task backend:build
task frontend:typecheck
task frontend:build
```

Run `wrangler dev` and `bun run dev` (or `task dev`) behind `cloudflared access login` to confirm Access headers flow correctly.

## 6. Deploy Backend (Worker + DO)

```bash
cd apps/backend
wrangler deploy --env preview         # add --env production when ready
wrangler tail --env preview           # optional smoke-test logs
```

- Confirms the Worker script, bindings, and Durable Object migrations are active.
- `task backend:deploy` wraps the same workflow once configuration is stable.

## 7. Deploy Frontend (Pages)

```bash
cd apps/frontend
task build
wrangler pages deploy dist --project-name=saavy-uptime --branch=preview
```

- Add custom domains with `wrangler pages project domain add saavy-uptime preview.example.com`.
- Use Access policies to guard the preview domain; public `/status/*` routes can bypass later.

## 8. Post-Deploy Smoke Checklist

1. **Access flow** – `cloudflared access curl https://preview.example.com/api/bootstrap/status`.
2. **Bootstrap** – Confirm `/api/bootstrap/status|initialize` work on preview DB.
3. **Monitor CRUD** – Create/update monitors via UI, verify D1 rows.
4. **Ticker + dispatch** – Tail logs to ensure DO alarms dispatch monitors and runner writes to `heartbeats`.
5. **AE ingest** – Check Analytics Engine dataset receives traces (use `wrangler analytics-engine query` once bindings exist).
6. **Status page** – Hit `/status/<slug>` (still Access-protected for now).

## 9. Demo Data (optional)

```bash
wrangler d1 execute saavy_uptime_preview --file apps/backend/seed/monitors.sql
# or
curl -X POST https://<preview-api>/api/internal/seed \
  -H "CF_Authorization: $(cloudflared access token --app ...)" \
  -d '{"orgId":"<preview-org-id>"}'
```

Update `{{ORG_ID}}` inside `apps/backend/seed/monitors.sql` before running the SQL batch.

## 10. Observability & Logs

- Use `wrangler tail` to stream structured logs (`console_error!`) from the Worker and Durable Object.
- Enable Analytics Engine dashboards (see `docs/observability.md`) by pointing Grafana to the AE dataset you provisioned.
- Capture dispatch IDs + `cf.colo` metadata early so future visualizations have data even in preview.

## 11. Toward One-Click Deploys

Short term:

1. **Taskfile orchestration** – `task deploy` already chains `frontend:deploy` + `backend:deploy`. Document required env vars so CI or humans can run it after `wrangler login`.
2. **Wrapper CLI** – Package the steps above behind `npx saavy-uptime@deploy` (or a Rust CLI) that:
   - Ensures prerequisites (`wrangler`, `task`, `cloudflared`) exist.
   - Prompts for Cloudflare account + resource names.
   - Writes the IDs into `wrangler.toml`.
   - Runs `task deploy`.
3. **GitHub Actions workflow** – Add a manual-dispatch workflow that installs toolchains, injects `CLOUDFLARE_API_TOKEN`, and runs `task deploy`. Acts as the bridge until Terraform owns infra.

Mid term (Terraform):

- Codify resource creation (Workers script, DO migration, D1, AE, R2, Access, Pages, DNS) so onboarding is `terraform apply` + `task deploy`.
- Surface variables (`project_name`, `access_audience`, dataset names) identical to the `wrangler.toml` schema to avoid drift.

Future “single command” flow could look like:

```bash
pnpm dlx saavy-uptime-deploy --env preview \
  --account <CF_ACCOUNT_ID> \
  --project-name saavy-uptime
# internally runs terraform (optional) + task deploy
```

Documenting today’s manual sequence keeps us honest while designing that experience.

## 12. Terraform Mapping Backlog

| Manual Step | Terraform Resource (target) |
| --- | --- |
| Worker script + routes | `cloudflare_workers_script`, `cloudflare_workers_deployment`, `cloudflare_workers_route` |
| Durable Object namespace/migrations | `cloudflare_workers_kv_namespace` + migration inputs (or wrapper module once CF exposes DO resource) |
| D1 databases + migrations | `cloudflare_d1_database` + CI job invoking `wrangler d1 migrations apply` |
| Analytics Engine dataset | `cloudflare_analytics_engine_dataset` |
| R2 buckets | `cloudflare_r2_bucket` |
| Access app + policy | `cloudflare_access_application`, `cloudflare_access_policy` |
| Pages project + domains | `cloudflare_pages_project`, `cloudflare_pages_project_domain` |
| DNS | `cloudflare_record` (CNAME/AAAA for Worker + Pages) |

Treat this table as the source of truth when building infra modules—each row corresponds to the manual commands in sections 3–7.
