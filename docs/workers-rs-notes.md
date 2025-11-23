## Notes on workers-rs + Axum (WASM)

We’re capturing bits that weren’t obvious while porting the backend to the Axum-enabled Workers runtime. The Cloudflare team has asked for feedback, so here’s what burned time.

### Randomness + IDs

- `cuid2` (and most RNG consumers) need `getrandom` to work inside WASM. Add `getrandom = { version = "^0.3", features = ["wasm_js"] }` or the create-id call will panic at runtime.
- The `wasm_js` feature installs a JavaScript shim that pulls randomness from the host. That’s fine for Workers but has the documented drawback of panicking if the host can’t provide entropy—worth calling out in docs.

### Dates / timestamps

- `std::time::SystemTime::now()` works, but anything fancier (like chrono) won’t compile to WASM without extra shims. We currently lean on `js_sys::Date::now()` to stamp records, and we had to add `js_sys` explicitly for that. A blessed “Workers clock” helper would be convenient.

### Axum integration

- The `workers-rs` README hints at Axum support, but there’s no end‑to‑end example for “wrap Env/D1, add Send, expose Router as the fetch handler.” We had to piece together:
  - Wrap `Env` in `worker::send::SendWrapper` for Axum state.
  - Annotate every handler that touches Workers APIs with `#[worker::send]`.
  - Manually convert `worker::HttpRequest` into an Axum `Router` service (and remember to map the body type).
- A maintained example repo showing CORS layers, state, and extractors with D1 would save a ton of trial and error.

### Feature flags

- `axum` has most features disabled by default. For Workers, you must enable at least `features = ["json"]` in `Cargo.toml` to get `Json<T>` extractors working.
- Similarly, the `worker` crate needs `features = ["http", "axum", "d1"]` set or you’ll get confusing trait errors.

### Access (JWT) verification

- Access JWTs live behind `CF_Authorization`/`Cf-Access-Jwt-Assertion` headers and use RS256. We had to roll our own verifier:
  - Fetch JWKS from `https://<team>.cloudflareaccess.com/cdn-cgi/access/certs`, cache it per-team (the cache survives while the Worker is warm).
  - Use the `rsa` crate (`rs256` isn’t provided by workers-rs yet) to verify `header.payload` with the JWK’s `n`/`e`.
  - Validate `aud`, `iss`, and `exp` manually since there’s no Workers helper.
- It would be great to have a first-class helper (or documented example) that hides the JWKS fetch + verification ceremony.

### Misc WASM constraints

- Using `rand`/`cuid` means you can’t rely on OS randomness. That ties back to `getrandom` but also means testing outside Workers should mock ID generation to avoid differences.
- Native TLS/verifier crates won’t work; stick to HTTP clients that support WASM (or call back through Fetch).
- Durable Object storage is eventually consistent across restarts; don’t assume `set_alarm` survives long sleeps unless you persist state yourself.

- **Durable Object tooling:** the macro/lifecycle story (`#[durable_object]`, `State`, `Env`) is solid, but authoring a DO plus Axum API in one crate lacks docs. We ended up reading `worker` crate source to understand how alarms, storage, and `Fetch::Request` interact.
- **Fetch within DO:** sending requests from a DO back into the Worker requires manually building `RequestInit`, headers, etc. Examples for “DO calling same Worker script via internal auth” would be helpful; today you have to inspect `worker::request` internals or rely on trial/error.
- **wasm-bindgen stack traces:** when fetch/set_alarm fail at runtime, the errors bubble up as `JsValue` strings (`Error: bad resource name`). There’s no direct mapping to Rust error types, so we wrap everything with `internal_error()` helpers to keep context. A higher-level error enum from workers-rs would be nice.
- **Lack of DO migrations doc:** Wrangler requires `[[migrations]]` entries for new DO classes, but the recommended pattern (“append migration per change”) isn’t mentioned in workers-rs docs—you learn it from Wrangler errors.
- **Local dev errors:** running `cargo check --target wasm32-unknown-unknown` can fail on filesystems that don’t support hardlinks (common on bind mounts). Not workers-rs’ fault, but worth calling out in docs so folks set `CARGO_TARGET_DIR` appropriately.
- **UDP support is still aspirational:** Cloudflare announced Socket Workers in 2021 with UDP examples, but (as of Nov 2025) only outbound TCP via `connect()` has shipped. UDP remains “not available,” despite the docs snippet. Track progress via <https://community.cloudflare.com/tag/socket-workers>. For now, any UDP-heavy protocols (Minecraft query, Valve A2S) need a proxy or alternative transport.
### Secret Store + Wrangler dev gotchas

- Secret Stores work great in production, but Wrangler’s local dev **does not** pull remote secrets. You must:
  1. Create the secret locally (omit `--remote`): `wrangler secrets-store secret create <store-id> AE_API_TOKEN`.
  2. Start dev with the store mounted: `wrangler dev --secrets-store <store-id>` so Miniflare hydrates the binding.
- Without step (2) you’ll see `Binding cannot be cast to the type String from Fetcher` when calling `env.secret("...")` / `env.secret_store("...")`.
- When using secret stores from workers-rs, call `env.secret_store("BINDING")?.get().await?`; the binding→secret mapping is taken from `secrets_store_secrets` in `wrangler.toml`, so `.get()` doesn’t take a name.

- **Analytics Engine bindings missing:** There’s no first-class AE binding in workers-rs; both reads and writes require manual `fetch` calls to `client/v4/accounts/{ACCOUNT_ID}/analytics_engine/sql` with API tokens. We’re considering upstreaming a binding layer—until then every project has to roll its own AE client.

### D1 ergonomics (parameter binding)

- Writing raw SQL against D1 is fine until you mix positional (`?1`) and anonymous (`?`) placeholders. The compiler can’t help you—if you bind values in the wrong order you get silent “0 rows updated” or `Wrong number of parameter bindings` at runtime. A few ideas that would improve DX:
  - **Linter/static check:** anything that inspects the SQL string + `bind(&[...])` call and warns when the parameter counts don’t line up (or when you use positional markers but pass anonymous binds).
  - **Query builder helper:** even a lightweight macro like `d1::update("monitors").set(("name", value)).where(("id", monitor_id))` would eliminate the manual string formatting and keep bindings aligned.
- **Better error surface:** today you get `DbBind`/`DbRun` with the raw Workers error, which often just says “Wrong number of parameter bindings.” Including the SQL string and placeholder count in the error would make debugging faster.
- **Testing utility:** an in-memory D1 adapter we can unit-test against (with asserts on bindings) would catch these mistakes before deploying to Workers.
 Until we have tooling, we’re triple-checking bind order manually and leaving comments near each `prepare` call. Anything workers-rs can do (even doc guidance) would save folks from hard-to-spot bugs.

### Queues batch config

- Queue consumers require `max_batch_size` / `max_batch_timeout` in `wrangler.toml`. That’s fine for static deployments, but we’d like a runtime knob so users can tune AE batch sizes without redeploying. Example: hobby orgs might want smaller batches to reduce latency, while larger tenants want 500+ messages per batch for cost efficiency. Right now the workaround is to set Wrangler’s limits high and enforce our own cap inside the consumer (via env vars), but a per-queue API or dynamic configuration would make operator UX nicer.

### External entrypoints and tooling

- Durable Object and Queue consumer entrypoints live in the repo but aren’t part of the main crate (they’re compiled via the `#[event(...)]` macros), so Rust Analyzer / cargo check often skip them. IDEs treat those files as “dead code,” meaning no diagnostics or inline errors until we run `worker-build`. It would be great if workers-rs exposed a virtual crate or recommended stub modules so tools can index these entrypoints. Until then we have to run `cargo check --lib --bin whatever` manually or accept that the analyzer can’t see them.
- Possible workaround: emit a `rust-project.json` (see <https://rust-analyzer.github.io/book/non_cargo_based_projects.html>) that tells rust-analyzer about the extra “crates” for DOs/queue consumers. We haven’t wired this up yet, but it might be worth generating one during build to keep IDEs happy.

If you run into other sharp edges, add them here so we can hand actionable notes back to the Workers team (or automate them via templates/Taskfile steps).
