# Docs site (VitePress + in-app hosting)

Bastion uses **VitePress** for documentation.

## Source and output

- Sources: `docs/`
- Build output: `docs/.vitepress/dist/`

## Base path (`/docs/` vs GitHub Pages)

The docs site base can be overridden via `DOCS_BASE`.

For in-app docs (served by the Hub), build with:

```bash
DOCS_BASE=/docs/ npm run build --prefix docs
```

## Serving docs from the Hub

The Hub serves documentation under:

- `/docs/` (public by default)

Modes:

- **Filesystem mode** (default dev): serves files from `docs/.vitepress/dist/`
  - Override with `BASTION_DOCS_DIR=/path/to/dist`
- **Embedded mode**: compile docs into the binary with `--features embed-docs` (or `embed-web`)
  - Requires the docs build output to exist at compile time.

## UI integration

The Web UI includes a **Help** entry that opens `/docs/` in a new tab.

When developing the UI (`npm run dev --prefix ui`), the dev server proxies `/docs/*` to the Hub so the Help link works.

