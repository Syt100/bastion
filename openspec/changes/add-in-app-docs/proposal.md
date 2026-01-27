# Change: Serve embedded product documentation under /docs

## Why
To improve onboarding and offline/self-hosted usability, Bastion should ship its documentation with the product and serve it directly from the Hub. This avoids relying on GitHub Pages and keeps docs aligned with the shipped binary.

## What Changes
- Build the docs site with VitePress and serve it from the Hub at `/docs/` (public by default).
- Support `embed-docs` so the docs site can be bundled into the Hub binary (similar to `embed-ui`).
- Add a Web UI entry point ("Help") that opens `/docs/`.
- Update CI/release build steps to build docs when needed.

## Impact
- Affected specs: `control-plane`, `web-ui`, `dev-workflow`
- Affected code: `crates/bastion-http/*`, `crates/bastion/*`, `scripts/*`, `ui/*`

## Non-Goals
- Documentation versioning.
- Search engine indexing controls.
- Replacing UI inline help text; this is a complementary help surface.

