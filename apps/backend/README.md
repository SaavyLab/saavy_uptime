## Backend (Rust Workers + Axum)

Rust 2021 Cloudflare Worker that exposes an Axum API, D1-backed persistence, and Access-protected routes.

### Prerequisites

- Rust stable toolchain
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `cargo install worker-build`
- Wrangler 3+
- `cloudflared` (for Access-authenticated local testing)

### Environment Setup

1. Install deps: `pnpm install` (for shared tooling) and `cargo fetch`.
2. Configure `wrangler.toml` with your D1/AE/R2 IDs **and** the Access values `ACCESS_TEAM_DOMAIN` + `ACCESS_AUD` (audience from the Access dashboard). Each Wrangler environment can override them if needed.
3. Run migrations: `wrangler d1 migrations apply saavy_uptime_dev`.
4. If hitting Access locally, log in and export a token:
   ```bash
   cloudflared access login https://your-app.example.com
   export CF_ACCESS_TOKEN=$(cloudflared access token --app https://your-app.example.com)
   ```

### Common Commands

Prefer the Taskfile wrappers to keep workflows consistent:

```bash
cd apps/backend

task backend:install    # cargo fetch
task backend:check
task backend:build
task backend:migrate DATABASE_NAME=saavy_uptime_dev
task backend:dev        # wrangler dev
task backend:deploy     # wrangler deploy
task backend:format
task backend:test
task backend:clean
```

Lower-level `cargo fmt`, `cargo test`, `wrangler dev`, etc., still work when you need to run them directly.

### Axum + Workers Notes

- Entry point: `src/lib.rs` converts the Workers `HttpRequest` into Axum’s router and returns an Axum response body.
- State: `src/router.rs` defines `AppState` (wrapping `Env` in `SendWrapper`) so Axum handlers can access D1 bindings.
- Handlers live under `src/monitors/**`, `src/organizations/**`, etc., and use `#[worker::send]` to satisfy Axum’s `Send` requirements inside Workers’ single-threaded runtime.
- `utils/wasm_types.rs` holds helpers for building `JsValue` bindings for D1 inserts.

### Cloudflare Access

When running locally, the frontend can inject `CF_Authorization` via `VITE_CF_ACCESS_TOKEN`. The backend verifies every token via `auth::jwt` (JWKS fetch + RS256 verification). Handlers that need identity can accept the `CurrentUser` extractor to get the decoded claims. Ensure your Access application/policy includes the dev accounts you use.

For deployment prerequisites and Terraform plans see the repository root `README.md`.
