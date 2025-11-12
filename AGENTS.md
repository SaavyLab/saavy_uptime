# AGENTS.md

Guidance for AI agents (and future contributors) working in this repo.

## Project Layout

- `apps/backend` – Rust Worker (Axum) with D1 + Access auth.
  - `src/auth/` – JWT verification, current user extractor, membership lookup.
  - `src/bootstrap/` – onboarding endpoints (`/api/bootstrap/status|initialize`).
  - `src/monitors/` – monitor handlers/services/types.
- `apps/frontend` – Vite/React dashboard.
  - Uses TanStack Router + Query, our custom form hooks, shadcn UI components, and Sonner for toasts.
- `docs/` – design docs, including `implementation-plan.md` and `workers-rs-notes.md`.

## Tooling & Commands

- Install prerequisites: Rust stable, `wasm32-unknown-unknown` target, `cargo install worker-build`, Wrangler v3, Node 20+, PNPM, `cloudflared`, and Task (`go install github.com/go-task/task/v3/cmd/task@latest`).
- Prefer Taskfile targets:
  - `task frontend:install`, `task frontend:dev`, `task frontend:build`, `task frontend:lint`.
  - `task backend:install`, `task backend:dev`, `task backend:build`, `task backend:check`.
- Direct commands when needed:
  - Backend: `cargo fmt`, `cargo clippy`, `wrangler dev`.
  - Frontend: `pnpm dev`, `pnpm lint`, `pnpm build`.
- D1 migrations live in `apps/backend/migrations`. Apply via `wrangler d1 migrations apply <db>`.

## Auth & Bootstrap

- Cloudflare Access is required. Locally, run `cloudflared access login https://<your-app>` and export `VITE_CF_ACCESS_TOKEN` before `pnpm dev`.
- Backend expects `ACCESS_TEAM_DOMAIN` and `ACCESS_AUD` bindings in `wrangler.toml`.
- The bootstrap flow:
  - `GET /api/bootstrap/status` → `{ isBootstrapped, suggestedSlug, email }`.
  - `POST /api/bootstrap/initialize` → creates the first org + member via D1 batch.
  - Frontend’s `BootstrapGate` blocks the app until status is true; `BootstrapWizard` hits `/initialize`.
- All authenticated routes use `CurrentUser` (and, when needed, membership lookup) to resolve the caller’s organization.

## Monitors

- Handlers live in `apps/backend/src/monitors/handlers.rs`.
- Service layer resolves membership via `auth::membership::load_membership` and only operates on the caller’s org.
- Frontend monitor APIs (`apps/frontend/src/lib/monitors.ts`) no longer accept an `orgId`; the backend derives it.

## Logging & Errors

- Use `console_error!` for structured error messages; see helper in `monitors/service.rs` and `bootstrap/handlers.rs`.
- Consider promoting the `internal_error` helper to a shared module if you touch additional services.

## Testing / Linting

- Backend: `cargo fmt`, `cargo clippy`, and `cargo check` (outside sandbox if necessary). There are no unit tests yet.
- Frontend: `pnpm lint` (Biome) and `pnpm test` (Vitest) when tests are added.

## Common Pitfalls

- Don’t trust client-provided `orgId`. Always resolve via membership.
- Access JWT verification requires the JWKS fetch; avoid touching headers/cookies manually—use `CurrentUser`.
- When running locally, remember to set `VITE_API_URL`/`VITE_CF_ACCESS_TOKEN` in `.env.local`.
- Use the new shadcn `Card` component (`src/components/ui/card.tsx`) for consistent styling.

## Next Steps (per implementation plan)

- Finish the monitor CRUD polish and build the scheduler (Phase 1 deliverable).
- Prepare for Phase 2 by sketching heartbeat storage + incident state machine.
- Add structured logging helper + potential `CurrentOrg` extractor using membership.

Refer to `docs/implementation-plan.md` for the broader roadmap before starting new work.
