## 1. Spec
- [x] 1.1 Add `hub-agent` spec delta for pending-request cleanup on agent disconnect
- [x] 1.2 Run `openspec validate optimize-agent-manager-pending-cleanup --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Hub/Agent - pending request cleanup
- [x] 2.1 Refactor `AgentManager::unregister` to drain pending maps without temporary key vectors
- [x] 2.2 Ensure all pending request types return explicit "agent disconnected/offline" errors (including artifact stream open)
- [x] 2.3 Add regression tests for pending cleanup behavior
- [x] 2.4 Run `cargo test --workspace`
- [x] 2.5 Commit AgentManager cleanup changes (detailed message)
