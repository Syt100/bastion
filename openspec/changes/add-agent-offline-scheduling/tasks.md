## 1. Spec
- [x] 1.1 Add spec deltas for Agent offline scheduling/execution and run sync behavior
- [x] 1.2 Run `openspec validate add-agent-offline-scheduling --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Hub ↔ Agent Config Sync
- [x] 2.1 Define protocol messages for config snapshot and acknowledgements
- [x] 2.2 Hub: compute per-agent config snapshots and send when changed
- [x] 2.3 Agent: persist snapshots locally (including encrypted-at-rest credential storage)
- [x] 2.4 Add/adjust tests for snapshot generation and persistence

## 3. Agent Offline Scheduler
- [x] 3.1 Agent: local scheduler triggers cached jobs on schedule when Hub is unreachable
- [x] 3.2 Agent: enforce overlap policy locally (reject/queue) consistent with Hub behavior
- [x] 3.3 Add/adjust tests for scheduling behavior

## 4. Run History Sync Back to Hub
- [x] 4.1 Agent: persist run records and events locally
- [x] 4.2 Agent: on reconnect, upload missing runs/events to the Hub
- [x] 4.3 Hub: ingest uploaded runs/events and surface them in UI
- [x] 4.4 Add/adjust tests for run ingestion and deduplication

## 5. Web UI
- [x] 5.1 Indicate when runs were executed offline (optional annotation)
- [x] 5.2 Ensure per-node UX remains consistent when runs arrive late

## 6. Validation
- [x] 6.1 Run backend checks (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] 6.2 Run UI checks (`npm run lint --prefix ui`, `npm test --prefix ui`, `npm run build --prefix ui`)

## 7. Commits
- [x] 7.1 Commit hub/agent protocol + offline scheduling (detailed message)
- [x] 7.2 Commit UI annotations (detailed message)
