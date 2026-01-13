# Bastion Web UI

Vue 3 + Vite single-page app for the Bastion Hub.

## Prerequisites

- Node.js `20.19+` or `22.12+` (see `package.json`)

## Install

From the repo root:

```sh
npm ci --prefix ui
```

## Development

The dev server proxies the Hub API and WebSocket endpoints to `http://127.0.0.1:9876` (see `vite.config.ts`).

Terminal 1 (Hub):

```sh
cargo run -p bastion
```

Terminal 2 (UI):

```sh
npm run dev --prefix ui
```

Open `http://localhost:5173`.

## Build

```sh
npm run build --prefix ui
```

Build output goes to `ui/dist`.

### Serving UI via the Hub

- **Filesystem mode** (default): the Hub serves UI assets from `./ui/dist` (relative to its working directory). You can override via `BASTION_UI_DIR=/path/to/dist`.
- **Embedded mode**: build the UI, then build/run the Hub with `--features embed-ui` to bake `ui/dist` into the binary.

## Tests / lint / type-check

```sh
npm test --prefix ui
npm run lint --prefix ui
npm run type-check --prefix ui
```
