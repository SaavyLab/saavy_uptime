## Frontend (Vite + React)

Cloudflare Pages-ready dashboard built with Vite, React 19, TanStack Router, TanStack Query/Form/Table, Tailwind v4, and shadcn-inspired components.

### Prerequisites

- Node 20+
- [PNPM](https://pnpm.io/) (preferred) or npm
- `cloudflared` if you want the dev server to hit an Access-protected backend

Create `.env` (or `.env.local`) with at least:

```bash
VITE_API_URL=http://localhost:8787
# Optional during local Access testing:
# VITE_CF_ACCESS_TOKEN=$(cloudflared access token --app https://your-app.example.com)
```

### Common Scripts

Use [`Taskfile`](../../Taskfile.yaml) to keep frontend/backed commands in sync:

```bash
task frontend:install
task frontend:dev
task frontend:build
task frontend:format
task frontend:test  # if added
```

Direct PNPM scripts are still available when needed:

```bash
pnpm install
pnpm dev
pnpm build
pnpm serve
pnpm test
pnpm lint
pnpm format
pnpm format:fix
pnpm check
```

### Cloudflare Access During Dev

If your backend Worker is behind Cloudflare Access, obtain a token before `pnpm dev`:

```bash
cloudflared access login https://your-app.example.com
export VITE_CF_ACCESS_TOKEN=$(cloudflared access token --app https://your-app.example.com)
pnpm dev
```

The fetch clients in `src/lib/*` automatically add `CF_Authorization` when `VITE_CF_ACCESS_TOKEN` exists. In production the browser already holds the Access cookie, so this env var should be unset.

### Project Notes

- Routing lives under `src/routes/**` using TanStack Router code-based routes.
- API helpers reside in `src/lib/` (monitors, organizations, etc.).
- UI primitives/components live in `src/components`.
- Environment validation uses `@t3-oss/env-core` (`src/env.ts`).

Refer to the root `README.md` for global architecture, backend instructions, and Terraform plans.
