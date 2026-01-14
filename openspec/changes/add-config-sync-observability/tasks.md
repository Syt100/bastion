## 1. Spec
- [x] 1.1 Add spec deltas for config sync observability + sync-now actions
- [x] 1.2 Run `openspec validate add-config-sync-observability --type change --strict`

## 2. Backend
- [ ] 2.1 Storage: add schema for desired/applied snapshot ids + last sync error
- [ ] 2.2 HTTP/WS: persist ConfigAck and update applied snapshot state
- [ ] 2.3 Backend: update desired snapshot id on snapshot send / config change
- [ ] 2.4 HTTP: include sync status in agents list/detail APIs
- [ ] 2.5 Bulk ops: add action “sync config now” (uses bulk framework)
- [ ] 2.6 Add backend unit tests for ack persistence and offline behavior

## 3. Web UI
- [ ] 3.1 Agents page: surface sync status (quick indicators + tooltip)
- [ ] 3.2 Agent details: show desired/applied ids and last error
- [ ] 3.3 Add “sync now” actions (single + bulk)
- [ ] 3.4 Add/adjust unit tests

## 4. Validation
- [ ] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [ ] 5.1 Commit spec proposal (detailed message)
- [ ] 5.2 Commit implementation (detailed message)
- [ ] 5.3 Mark tasks complete and commit
