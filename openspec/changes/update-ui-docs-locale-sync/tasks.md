---
## 1. Spec
- [x] 1.1 Add `control-plane` spec delta for language-aware docs entrypoint redirects and locale cookie
- [x] 1.2 Add `web-ui` spec delta for unified locale resolution + docs entrypoint locale sync
- [x] 1.3 Run `openspec validate update-ui-docs-locale-sync --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Add shared locale cookie helpers in the Hub docs server (`bastion-http`) and implement `/docs` + `/docs/` locale redirect
- [ ] 2.2 Set `bastion_locale` cookie on served docs HTML responses based on requested locale path
- [ ] 2.3 Update Web UI locale resolution (localStorage → cookie → browser → default `en-US`) and persist to cookie
- [ ] 2.4 Update Web UI "Help" entry to open localized docs root (`/docs/` vs `/docs/zh/`)

## 3. Tests / Validation
- [ ] 3.1 Add backend tests for `/docs` and `/docs/` locale redirect behavior (query param, cookie, Accept-Language, default)
- [ ] 3.2 Add frontend unit tests for initial locale resolution priority order
- [ ] 3.3 Run `bash scripts/ci.sh`

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit
