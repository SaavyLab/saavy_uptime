# AGENTS.md

Guidance for AI agents (and future contributors) working in this repo.

## The "hb" Stack

We are building `hb`, a collection of sharp, single-purpose Rust crates for Cloudflare Workers (see `crates/hb-README.md`).
Current components:
- `hb-d1c`: Type-safe SQL generation.
- `hb-tracing`: Distributed tracing to Analytics Engine.
- `hb-auth`: Access JWT validation & permission DSL.
- `hb-sync`: Distributed primitives (Mutex, RWLock) for Durable Objects.

**Goal:** As we build `saavy-uptime`, identify reusable patterns (e.g., feature flags, secret management, structured logging) that should be extracted into new `hb-*` crates.

## Project Layout

- `apps/backend` – Rust Worker (Axum) with D1 + Access auth.
- `apps/frontend` – Vite/React dashboard (TanStack Router/Query, shadcn).
- `crates/` – The `hb` stack crates.

## Database & Queries (hb-d1c)

We use `hb-d1c` (aliased as `d1c`) for type-safe D1 queries.

**The Workflow:**
1.  **Write SQL** in `apps/backend/db/queries/*.sql`.
2.  `d1c` (running in watch mode) **automatically updates** `apps/backend/src/d1c/*.rs`.
3.  **Import and use** the generated functions in your Rust code.

**Critical Rules for Agents:**
-   **NEVER** edit files in `apps/backend/src/d1c/` manually. They are machine-generated.
-   **NEVER** run `d1c generate`. Assume the user has `d1c watch` running.
-   **ALWAYS** follow `crates/hb-d1c/QUERY_FORMAT.md` (e.g., `-- name: MyQuery :one`).
-   **ALWAYS** use named parameters (`:id`), never positional (`?1`).

## Core Workflows

### Auth & Bootstrapping
- **Auth:** Cloudflare Access is required. Backend expects `ACCESS_TEAM_DOMAIN`/`ACCESS_AUD`.
- **Bootstrap:** `POST /api/bootstrap/initialize` creates the first org/user. Frontend blocks on `GET /status` until true.
- **Context:** All auth'd routes use `CurrentUser` extractor.

### Monitors
- **Logic:** `apps/backend/src/monitors/`.
- **Org Scoping:** Never confiding in client `orgId`. Always derive from `CurrentUser` membership.

## Tooling

- **Taskfile:** Use `task backend:dev`, `task frontend:dev`.
- **Migrations:** `apps/backend/migrations`. Apply with `wrangler d1 migrations apply`.
- **Logs:** Use `console_error!` for structured errors.

## "hb" Extraction Candidates

Be on the lookout for:
- **Feature Flags:** We need a strongly-typed flag system backed by KV/D1.
- **Secrets:** A compile-time validated secret loader to prevent runtime panics.
- **Rate Limiting:** Distributed rate limiters for DOs.

