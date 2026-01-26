# Change: Add VitePress documentation site (GitHub Pages, /docs)

## Why
For an open-source project, a searchable documentation site dramatically improves discoverability, onboarding, and long-term maintainability compared to browsing Markdown files in the repo.

## What Changes
- Add a VitePress-based documentation site powered by the existing `docs/` Markdown content.
- Publish the site via GitHub Pages, with the public entry point at `/<repo>/docs/`.
- Keep docs unversioned initially (single "latest" docs set).

## Impact
- Affected specs: `dev-workflow`
- Affected code: `docs/*`, `.github/workflows/*`, `.gitignore`

## Non-Goals
- Documentation versioning per release/tag.
- Custom domain configuration (can be added later).
- Major content rewrites of existing Markdown docs.

