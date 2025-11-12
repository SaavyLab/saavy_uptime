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

If you run into other sharp edges, add them here so we can hand actionable notes back to the Workers team (or automate them via templates/Taskfile steps).
