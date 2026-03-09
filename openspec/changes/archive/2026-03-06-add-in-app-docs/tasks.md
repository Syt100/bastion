## 1. Spec
- [x] 1.1 Add `control-plane` spec delta for serving docs at `/docs/` (public by default)
- [x] 1.2 Add `web-ui` spec delta for a "Help" entry point linking to `/docs/`
- [x] 1.3 Add `dev-workflow` spec delta for building docs in CI/release as needed
- [x] 1.4 Run `openspec validate add-in-app-docs --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add docs static server in `bastion-http` and mount routes `/docs` (redirect) and `/docs/*path`
- [x] 2.2 Add `embed-docs` feature (filesystem mode uses `BASTION_DOCS_DIR`, default `docs/.vitepress/dist`)
- [x] 2.3 Update `bastion` crate features to expose `embed-docs` (and optionally a combined feature)
- [x] 2.4 Add Web UI "Help" button/menu item that opens `/docs/`
- [x] 2.5 Update CI script and release workflow to build docs with `DOCS_BASE=/docs/` before `embed-docs` builds

## 3. Tests / Validation
- [x] 3.1 Add backend tests covering `/docs` redirect, `/docs/` serving, and path traversal rejection
- [x] 3.2 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [x] 4.2 Mark OpenSpec tasks complete and commit
