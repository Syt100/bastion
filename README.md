# Bastion

Bastion is a self-hosted backup orchestrator (Hub + optional Agents) with a Web UI.

Documentation:

- In-app (served by the Hub): `/docs/`
- Source files: `docs/` (EN) and `docs/zh/` (ZH)

This project is early-stage. Breaking changes may occur before the first stable release.

## Components

- **Hub** (this repo’s main binary): HTTP API + Web UI, scheduling, metadata storage (SQLite), secrets management.
- **Agent** (Hub subcommand): connects to the Hub over WebSocket and executes jobs on remote nodes.
- **Web UI** (`ui/`): Vue 3 + Vite single-page app.

## Key features

- **Agent labels**: tag agents (e.g., `prod`, `cn`, `db`) and filter/target operations by label. See `docs/user/agents.md`.
- **Config sync observability**: each agent shows desired/applied config snapshot state and last sync error, with “Sync now” actions. See `docs/user/agents.md`.
- **Bulk operations**: create async, per-agent operations with progress tracking + retry/cancel. See `docs/user/bulk-operations.md`.
  - Bulk labels add/remove
  - Bulk “sync config now”
  - Bulk WebDAV credential distribution from Hub to agents
  - Bulk deploy (clone) a job to many agents
- **Job scheduling**: manual/simple/cron schedules with explicit schedule timezone and overlap policy. See `docs/user/jobs.md`.

## Quickstart (local dev)

### Prerequisites

- Rust `1.92+` (workspace `rust-version = 1.92`)
- Node.js `20.19+` or `22.12+` (see `ui/package.json`)

### Run Hub (backend)

```bash
cargo run -p bastion
```

The Hub listens on `127.0.0.1:9876` by default.

> HTTPS is enforced for non-loopback traffic. For LAN/dev, either:
> - keep Hub bound to loopback, or
> - run with `--insecure-http`, or
> - put it behind a reverse proxy and configure trusted proxies.

### Run Web UI in dev mode (recommended for UI development)

Terminal 1:

```bash
cargo run -p bastion
```

Terminal 2:

```bash
npm ci --prefix ui
npm run dev --prefix ui
```

Open the UI at `http://localhost:5173`. The dev server proxies:
- `/api/*` → `http://127.0.0.1:9876`
- `/agent/*` → `http://127.0.0.1:9876` (including WebSocket upgrade)
- `/docs/*` → `http://127.0.0.1:9876` (product docs, built output)

To build docs for the Hub (filesystem mode):

```bash
npm ci --prefix docs
npm run build --prefix docs
```

Then open `http://localhost:5173/docs/` (or use the UI "Help" button).

### Serve the built UI from the Hub (filesystem mode)

```bash
npm ci --prefix ui
npm run build --prefix ui

# Hub serves from ./ui/dist by default when not using embed-ui
cargo run -p bastion
```

If you want to serve UI assets from a different directory, set `BASTION_UI_DIR` (only used in filesystem mode):

```bash
BASTION_UI_DIR=/path/to/ui/dist cargo run -p bastion
```

### Serve the built UI from the Hub (embedded mode)

Embedded mode bakes `ui/dist` into the Hub binary at build time.

```bash
npm ci --prefix ui
npm run build --prefix ui

cargo run -p bastion --features embed-ui
```

## First run: setup

On first launch, Bastion requires initialization (create the first user). Open the UI and complete the setup flow.

Health/System endpoints (always available, even when HTTPS is enforced):
- `GET /api/health`
- `GET /api/system`
- `GET /api/setup/status`

## Agent: enroll and connect

1. In the Web UI, create an **enrollment token** (Agents page).
2. Run the agent on the target machine:

```bash
bastion agent \
  --hub-url http://127.0.0.1:9876 \
  --enroll-token <token> \
  --name "<friendly-name>"
```

Agents store their enrollment identity in their own data directory (see `BASTION_DATA_DIR`).

## Configuration

Most settings can be configured via CLI flags or environment variables (CLI takes precedence).

Reference:

- Defaults + precedence: `docs/user/operations/defaults.md`
- Full CLI reference (generated): `docs/user/reference/cli.md`

Common Hub options:

- `--host` / `BASTION_HOST` (default `127.0.0.1`)
- `--port` / `BASTION_PORT` (default `9876`)
- `--data-dir` / `BASTION_DATA_DIR`
- `--insecure-http` / `BASTION_INSECURE_HTTP` (LAN/dev only)
- `--trusted-proxy` / `BASTION_TRUSTED_PROXIES` (repeatable / comma-separated CIDRs)

Common Agent options:

- `--hub-url` / `BASTION_HUB_URL`
- `--enroll-token` / `BASTION_AGENT_ENROLL_TOKEN` (first-time enrollment)
- `--name` / `BASTION_AGENT_NAME`
- `--data-dir` / `BASTION_DATA_DIR`

## Security notes

- Bastion stores state in a data directory (SQLite + encrypted secrets). See `docs/user/operations/data-directory.md`.
- Use keypack export/import to back up or migrate `master.key`. See `docs/user/operations/data-directory.md`.
- For reverse proxy deployments (TLS termination), ensure `X-Forwarded-*` headers are set and proxies are trusted. See `docs/user/operations/reverse-proxy.md`.

## Development

Run the repo’s full checks (roughly what CI runs):

```bash
bash scripts/ci.sh
```

### Optional: enable git hooks

This repo ships optional Git hooks under `.githooks/` (e.g. to prevent accidental literal `\n` in commit messages).

Enable locally:

```bash
git config core.hooksPath .githooks
```

This script includes a `gitleaks` secret scan step. If `gitleaks` is not installed, it will attempt to install a pinned version via `go install` (requires Go) into `~/.cache/bastion-tools/bin`.

GitHub Actions CI runs the same script on pushes and pull requests (see `.github/workflows/ci.yml`).

## Docs

- Product docs: `/docs/` (served by the Hub; can be embedded in release builds)
- `docs/README.md` — documentation index
- `docs/user/operations/data-directory.md` — data directory layout + key management
- `docs/user/operations/logging.md` — logging configuration + rotation
- `docs/user/operations/reverse-proxy.md` — reverse proxy examples (Nginx/Caddy)
- `docs/user/recipes/vaultwarden.md` — Vaultwarden backup recipe

## License

Apache-2.0. See `LICENSE`.
