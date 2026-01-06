## 1. Spec
- [ ] 1.1 Add spec deltas for Agent offline scheduling/execution and run sync behavior
- [ ] 1.2 Run `openspec validate add-agent-offline-scheduling --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Hub ↔ Agent Config Sync
- [ ] 2.1 Define protocol messages for config snapshot and acknowledgements
- [ ] 2.2 Hub: compute per-agent config snapshots and send when changed
- [ ] 2.3 Agent: persist snapshots locally (including encrypted-at-rest credential storage)
- [ ] 2.4 Add/adjust tests for snapshot generation and persistence

## 3. Agent Offline Scheduler
- [ ] 3.1 Agent: local scheduler triggers cached jobs on schedule when Hub is unreachable
- [ ] 3.2 Agent: enforce overlap policy locally (reject/queue) consistent with Hub behavior
- [ ] 3.3 Add/adjust tests for scheduling behavior

## 4. Run History Sync Back to Hub
- [ ] 4.1 Agent: persist run records and events locally
- [ ] 4.2 Agent: on reconnect, upload missing runs/events to the Hub
- [ ] 4.3 Hub: ingest uploaded runs/events and surface them in UI
- [ ] 4.4 Add/adjust tests for run ingestion and deduplication

## 5. Web UI
- [ ] 5.1 Indicate when runs were executed offline (optional annotation)
- [ ] 5.2 Ensure per-node UX remains consistent when runs arrive late

## 6. Validation
- [ ] 6.1 Run backend checks (`cargo fmt`, `cargo clippy`, `cargo test`)
- [ ] 6.2 Run UI checks (`npm run lint --prefix ui`, `npm test --prefix ui`, `npm run build --prefix ui`)

## 7. Commits
- [ ] 7.1 Commit hub/agent protocol + offline scheduling (detailed message)
- [ ] 7.2 Commit UI annotations (detailed message)

