## 1. Spec
- [ ] 1.1 Finalize backend/web-ui deltas for phase-2 error-envelope rollout
- [ ] 1.2 Run `openspec validate update-error-envelope-rollout-phase2 --strict`

## 2. Backend implementation
- [ ] 2.1 Add envelope emission for Agent-bridged snapshot-delete/task-result failure events
- [ ] 2.2 Add envelope emission for execute-stage failure/warn events (filesystem/sqlite/vaultwarden)
- [ ] 2.3 Preserve legacy fields in parallel for compatibility
- [ ] 2.4 Add regression tests for envelope shape, protocol metadata, and fallback synthesis

## 3. Web UI implementation
- [ ] 3.1 Update maintenance/snapshot diagnostics views to prefer envelope data when available
- [ ] 3.2 Keep legacy fallback rendering for task-level `last_error_kind/last_error`
- [ ] 3.3 Add i18n keys and UI tests for envelope-first + fallback behavior

## 4. Validation and release notes
- [ ] 4.1 Run targeted Rust/UI tests for touched modules
- [ ] 4.2 Run `scripts/ci.sh`
- [ ] 4.3 Update `CHANGELOG.md` for user-visible diagnostics improvements
