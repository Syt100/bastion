# Documentation

This directory contains the documentation sources.

- In-app docs entry: `/docs/`
- Entry page: `docs/index.md` (VitePress)

---

## Structure

- `docs/index.md` — landing page (user manual + developer docs)
- `docs/user/` — user manual (Web UI usage + operations for self-hosting)
- `docs/dev/` — developer docs (build/dev workflow, docs-site integration, architecture notes)

## Notes

- VitePress output is built into `docs/.vitepress/dist/`.
- The Hub serves docs from `docs/.vitepress/dist` by default (filesystem mode), or from embedded assets when built with `embed-docs`.
