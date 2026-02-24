## 1. Spec
- [x] 1.1 Create proposal/design/spec deltas for security + queue stability batch
- [x] 1.2 Run `openspec validate update-security-and-queue-stability --strict`

## 2. Dependabot remediation (P0)
- [x] 2.1 Replace vulnerable `glib` dependency path by switching Windows tray dependency to a Windows-only tray crate
- [x] 2.2 Regenerate/verify lockfile so alert `#7` is no longer reproducible
- [x] 2.3 Run targeted build checks for `bastion` crate

## 3. Offline bounded queue hardening (P1)
- [x] 3.1 Replace offline scheduler unbounded task channel with bounded channel
- [x] 3.2 Replace offline writer unbounded command channel with bounded channel and explicit full/closed handling
- [x] 3.3 Add/update regression tests for bounded queue behavior and inflight counter correctness

## 4. Notifications queue keyset pagination (P1)
- [x] 4.1 Add keyset listing support in storage layer ordered by `(created_at DESC, id DESC)`
- [x] 4.2 Extend HTTP queue API with optional opaque cursor (`next_cursor`) while preserving existing paging compatibility
- [x] 4.3 Add regression tests for cursor continuity and invalid cursor handling

## 5. Query/index fit improvements (P2)
- [x] 5.1 Add composite index for run artifacts keyset ordering/filter path
- [x] 5.2 Add composite index for notifications queue keyset ordering/filter path
- [x] 5.3 Validate migrations and relevant list tests

## 6. UI race/perf polish (P2)
- [x] 6.1 Make locale switching last-write-wins under rapid toggles
- [x] 6.2 Skip dashboard desktop-table prefetch when viewport is not desktop
- [x] 6.3 Add/update targeted UI tests

## 7. Final validation and release notes
- [x] 7.1 Run full checks (`scripts/ci.sh` when feasible)
- [x] 7.2 Update `CHANGELOG.md` for user-visible changes
- [x] 7.3 Finalize checklist status
