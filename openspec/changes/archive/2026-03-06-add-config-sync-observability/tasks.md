## 1. Spec
- [x] 1.1 Add spec deltas for config sync observability + sync-now actions
- [x] 1.2 Run `openspec validate add-config-sync-observability --type change --strict`

## 2. Backend
- [x] 2.1 Storage: add schema for desired/applied snapshot ids + last sync error
- [x] 2.2 HTTP/WS: persist ConfigAck and update applied snapshot state
- [x] 2.3 Backend: update desired snapshot id on snapshot send / config change
- [x] 2.4 HTTP: include sync status in agents list/detail APIs
- [x] 2.5 Bulk ops: add action “sync config now” (uses bulk framework)
- [x] 2.6 Add backend unit tests for ack persistence and offline behavior

## 3. Web UI
- [x] 3.1 Agents page: surface sync status (quick indicators + tooltip)
- [x] 3.2 Agent details: show desired/applied ids and last error
- [x] 3.3 Add “sync now” actions (single + bulk)
- [x] 3.4 Add/adjust unit tests

## 4. Validation
- [x] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit spec proposal (detailed message)
- [x] 5.2 Commit implementation (detailed message)
- [x] 5.3 Mark tasks complete and commit
