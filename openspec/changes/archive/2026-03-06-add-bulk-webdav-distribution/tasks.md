## 1. Spec
- [x] 1.1 Add spec deltas for bulk WebDAV distribution (preview + skip/overwrite)
- [x] 1.2 Run `openspec validate add-bulk-webdav-distribution --type change --strict`

## 2. Backend
- [x] 2.1 Bulk ops: add action “distribute webdav secret”
- [x] 2.2 Implement default skip and optional overwrite semantics
- [x] 2.3 Implement a preview capability for UI (dry-run plan)
- [x] 2.4 Ensure node config snapshot is refreshed / marked pending
- [x] 2.5 Add backend tests for copy/skip/overwrite and offline behavior

## 3. Web UI
- [x] 3.1 Hub storage page: add “Distribute to nodes” flow
- [x] 3.2 Add preview UI (per-node will-skip / will-update)
- [x] 3.3 Trigger bulk operation and link to bulk results
- [x] 3.4 Add/adjust unit tests

## 4. Validation
- [x] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit spec proposal (detailed message)
- [x] 5.2 Commit implementation (detailed message)
- [x] 5.3 Mark tasks complete and commit
