## 1. Spec
- [x] 1.1 Add spec deltas for docs/doctor/defaults/observability
- [x] 1.2 Run `openspec validate update-ops-docs-doctor-observability --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Docs: add Upgrade & rollback guide (EN/ZH)
- [ ] 2.2 Docs: clarify defaults + config precedence (EN/ZH)
- [ ] 2.3 CLI: extend `bastion doctor` checks + actionable guidance (keep `--json`)
- [ ] 2.4 Backend: add readiness endpoint + document liveness/readiness semantics

## 3. Validation
- [ ] 3.1 Run `bash scripts/ci.sh`
- [ ] 3.2 Validate via GitHub Actions (workflow_dispatch dry-run is OK)

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit
