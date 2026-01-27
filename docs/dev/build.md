# Build and run

## Prerequisites

- Rust `1.92+`
- Node.js `20.19+` or `22.12+`

## Run the Hub (backend)

```bash
cargo run -p bastion
```

Default bind: `127.0.0.1:9876`.

## Run the Web UI in dev mode

Terminal 1:

```bash
cargo run -p bastion
```

Terminal 2:

```bash
npm ci --prefix ui
npm run dev --prefix ui
```

Open `http://localhost:5173`.

The UI dev server proxies:

- `/api/*` → Hub
- `/agent/*` → Hub (WebSocket)
- `/docs/*` → Hub (in-app docs)

## Run the docs site in dev mode

```bash
npm ci --prefix docs

# VitePress uses 5173 by default; pick another port if the UI dev server is running.
npm run dev --prefix docs -- --port 5174
```

## Build assets for embedded builds

Build UI:

```bash
npm ci --prefix ui
npm run build-only --prefix ui
```

Build docs:

```bash
npm ci --prefix docs
DOCS_BASE=/docs/ npm run build --prefix docs
```

Then build the Hub with embedded assets:

```bash
cargo build -p bastion --features embed-web
```

Notes:

- `embed-web` = `embed-ui` + `embed-docs`.
- If you only want to embed one of them, use `--features embed-ui` or `--features embed-docs`.
