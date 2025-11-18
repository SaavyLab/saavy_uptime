# Saavy Uptime: Deployment Strategy & UX Goals

## 1. The Core Philosophy
**"Node.js Ease, Rust Performance."**

The biggest friction point for Rust on Cloudflare Workers is the toolchain setup. Our deployment strategy bypasses this entirely for the end-user.
- **Zero Local Rust Requirement:** A user can deploy the full stack without ever installing `cargo` or `rustc` locally.
- **GitOps Native:** The GitHub Repository is the source of truth. All changes to infrastructure (Queues, D1, Cron) happen via `git push`.
- **Progressive Enhancement:** We leverage the GitHub CLI (`gh`) for magic automation but provide a manual fallback path.

---

## 2. The User Journey (The "Happy Path")

**Prerequisites:** Node.js installed, GitHub CLI (`gh`) installed & logged in.

1.  **User runs one command:**
    ```bash
    npx @saavy/create-uptime
    ```
2.  **Interactive Wizard:**
    - *"Where should we create the project?"* → `my-status-page`
    - *"Which profile?"* → `Hobby (Free Tier)` vs `Production (High Perf)`
    - *"Deploy now?"* → `Yes`
    - *"Cloudflare Credentials?"* → (Secure input for Account ID / API Token)
3.  **Automated Magic (Backend):**
    - Repo scaffolded from template.
    - `wrangler.toml` patched with selected profile settings.
    - Private GitHub repo created.
    - Secrets (`CLOUDFLARE_API_TOKEN`, etc.) uploaded to GitHub.
    - `deploy.yml` workflow triggered.
4.  **Success State:**
    - User is given a link to the running GitHub Action.
    - 3 minutes later, the app is live on the Edge.

---

## 3. Technical Implementation

### A. The CLI: `@saavy/create-uptime`
A lightweight Node.js CLI that orchestrates the setup.

*   **Stack:** `prompts`, `tiged` (repo cloning), `execa` (running git/gh commands), `chalk`.
*   **Profile Logic:**
    The template `wrangler.toml` will contain placeholders. The CLI replaces them based on the user's choice.
    ```toml
    # Template
    [queues.consumers]
    max_batch_size = {{BATCH_SIZE}}
    max_batch_timeout = {{BATCH_TIMEOUT}}
    ```
    *   **Hobby:** Batch 10, Timeout 1s (Low latency, higher potential bill if high volume).
    *   **Pro:** Batch 100, Timeout 10s (High throughput, optimized for cost).
*   **GitHub Integration:**
    Uses `execa('gh', ...)` to handle authentication and API calls. This ensures Windows/Mac/Linux compatibility without us maintaining complex OAuth logic.

### B. The Repository Template (`saavy-org/saavy-uptime`)
The repo must be structured to support this "Headless" deployment.

*   **`wrangler.toml`**: The single source of truth.
*   **`d1c.toml`**: Configured to run generation relative to the crate root.
*   **`create-saavy-uptime.mjs`**: (Optional) We might bundle the setup script inside the repo too, allowing users to run `npm run setup` if they cloned manually instead of using `npx`.

### C. The CI/CD Pipeline (`.github/workflows/deploy.yml`)
This is the engine that replaces local Rust installation.

**Workflow Steps:**
1.  **Checkout:** Fetch code.
2.  **Install Rust:** Use `dtolnay/rust-toolchain` (stable).
3.  **Install Tools:**
    - `cargo install worker-build`
    - `cargo install d1c` (or run from local path if monorepo)
    - `npm install -g wrangler`
4.  **Code Generation:**
    - Run `d1c gen` (Ensures queries.rs matches the latest SQL).
    - *Note:* This prevents "stale generated code" bugs.
5.  **Infrastructure Sync:**
    - `wrangler d1 migrations apply DB --remote`
6.  **Deploy:**
    - `wrangler deploy` (Deploys Workers & Durable Objects).
    - `npm run build` & `wrangler pages deploy` (Deploys Frontend).

---

## 4. Day 2 Operations (GitOps)

How does the user manage the app after the initial `npx` command?

### Scenario A: Tuning Performance
*   **User Action:** Opens `wrangler.toml` in GitHub UI (or VS Code).
*   **Change:** Edits `max_batch_size = 50`.
*   **Deploy:** Commits and pushes.
*   **Result:** GitHub Action runs `wrangler deploy`. Cloudflare updates the queue consumer config.

### Scenario B: Modifying SQL
*   **User Action:** Edits `db/queries/monitors.sql`.
*   **Deploy:** Commits and pushes.
*   **Result:** GitHub Action runs `d1c gen`, compiles the new Rust code with updated types, and deploys.
    *   *Safety Net:* If the SQL is invalid, `d1c gen` fails in CI, stopping the deploy.

### Scenario C: Database Schema Changes
*   **User Action:** Adds `db/migrations/0002_add_column.sql`.
*   **Deploy:** Commits and pushes.
*   **Result:** GitHub Action runs `wrangler d1 migrations apply --remote` before deploying the worker.

---

## 5. Fallback Strategy (No `gh` CLI)

If the user does not have the GitHub CLI installed, `@saavy/create-uptime` will downgrade gracefully:

1.  Scaffold the folder locally.
2.  Apply the "Hobby/Pro" profile to `wrangler.toml`.
3.  Print **"Manual Deployment Instructions"**:
    > 1. Create a repo on GitHub.
    > 2. Push this code.
    > 3. Add `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` to Repository Secrets.
    > 4. Enable Actions.

---

## 6. Action Items

1.  **Build the Package:** Create `packages/create-saavy-uptime`.
2.  **Update CI:** Refine `deploy.yml` to include the `d1c gen` step (ensuring `d1c` is available in the runner).
3.  **Template Prep:** Tokenize the `wrangler.toml` in the main repo (or have the CLI support standard TOML parsing to update values cleanly without handlebars syntax).
4.  **Docs:** Write the "Getting Started" section of the README to highlight `npx @saavy/create-uptime` as the primary method.