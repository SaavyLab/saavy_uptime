# Deployment Guide

Living runbook for getting Saavy Uptime into any Cloudflare environment today while we work toward a true one-click experience driven by `wrangler.toml`, Taskfile helpers, and future Terraform modules.

## 0. Deploy Button Fast-Path (recommended)

Cloudflare’s Deploy Buttons remove most of the manual toil:

1. Click the **Deploy to Cloudflare** badge in `README.md` (or visit `https://deploy.workers.cloudflare.com/?url=https://github.com/SaavyLab/saavy_uptime`). Cloudflare will clone the repo into your GitHub/GitLab account.
2. Accept or edit the default resource names for the Worker, Durable Object namespace, D1 database, Analytics Engine dataset (heartbeats), R2 bucket, and Secrets Store. The placeholders in `wrangler.toml` are intentionally generic so the button can provision fresh resources.
3. Fill in the environment variable + secret prompts (Access team domain/audience, dispatch token, AE account ID, etc.). The descriptions defined in `package.json` surface here.
4. Confirm the build/deploy commands. The generated scripts run the frontend build, apply migrations via `wrangler d1 migrations apply DB`, then deploy the Worker. After the workflow completes you’ll have a fully provisioned stack (plus a fork you own).
5. Manual follow-ups: create/adjust your Cloudflare Access application to protect the Worker + Pages domain, add any custom domains/DNS, and backfill demo data if desired.

Use the remaining sections below when you need to understand the internals, reproduce the steps manually, or customize beyond what the button currently supports.

## 1. Tooling & Access

| Requirement | Notes |
| --- | --- |
| Cloudflare account + target zone | Need access to create Workers, Durable Objects, D1, AE, R2, Pages, and Access apps. |
| CLI utilities | `wrangler` ≥ v3, Rust stable + `wasm32-unknown-unknown`, `cargo install worker-build`, Node 20+/Bun, PNPM (or npm), `go-task`, `cloudflared`. |
| Auth | `wrangler login`, `cloudflared access login https://<app>` then export `VITE_CF_ACCESS_TOKEN` for local/preview smoke tests. |
| Secrets/config | `ACCESS_TEAM_DOMAIN`, `ACCESS_AUD`, `DISPATCH_TOKEN`, etc. live in `wrangler.toml` (per-env overrides encouraged). |

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
5. **AE ingest** – Check the Analytics Engine dataset receives heartbeat summaries (use `wrangler analytics-engine query` once bindings exist).
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
- Wire up your preferred dashboard tool (Grafana, etc.) directly to the AE dataset you provisioned; sample queries live in `docs/analytics-engine-plan.md`.
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

## CI Deploy Reality Check (GitHub Actions)

Wrangler still needs resource IDs baked into `wrangler.toml` (or an override). That means a “from zero with no prior IDs” run cannot be 100 % automatic. To make the provided `Deploy (Cloudflare)` workflow succeed:

1. **Secrets Store (one-time)** – `wrangler secrets-store store create <name>` then capture the ID (e.g., `8876bad33f...`). Add it to `wrangler.toml` under `secrets_store_secrets` for each env, or pass it to the workflow input `secrets_store_id`.
2. **Resource IDs in config** – Ensure `wrangler.toml` has the correct IDs/names for D1, AE, R2, and DO script names per environment. The workflow can create these if missing, but Wrangler still reads the IDs from config when deploying.
3. **AE token** – Place `AE_API_TOKEN` in GitHub secrets. The workflow will write it into the Cloudflare secrets store for the selected environment.
4. **Run the workflow** – Manually dispatch `Deploy (Cloudflare)` with `environment=preview` or `production`, optional `api_base_url` (otherwise it derives workers.dev for preview), and optional `provision_resources=true` to create missing Cloudflare resources.
5. **Still manual today** – Access app/policy and custom domains must be set in the Cloudflare UI/API. If you change IDs/domains, update `wrangler.toml` (or a `wrangler.ci.toml` override) before re-running CI.
