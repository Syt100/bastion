## 1. Spec
- [x] 1.1 Add spec deltas for Hub runtime config UI + API
- [x] 1.2 Run `openspec validate add-hub-runtime-config-ui --strict`

## 2. Backend
- [x] 2.1 Storage: add hub runtime config repo (settings JSON)
- [x] 2.2 Startup: load saved config and apply when CLI/ENV not set
- [x] 2.3 HTTP: add authenticated GET/PUT API with effective/saved/source
- [x] 2.4 Add backend unit tests for repo/validation

## 3. Web UI
- [ ] 3.1 Add store + API wiring
- [ ] 3.2 Add Settings page (restart banner + read-only + editable fields)
- [ ] 3.3 Add i18n strings and route/navigation entry
- [ ] 3.4 Add/adjust unit tests

## 4. Validation
- [ ] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit spec proposal (detailed message)
- [x] 5.2 Commit backend changes (detailed message)
- [ ] 5.3 Commit Web UI changes (detailed message)
- [ ] 5.4 Mark tasks complete and commit
