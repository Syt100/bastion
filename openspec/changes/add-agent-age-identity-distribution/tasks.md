---
## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`backend`, `hub-agent`, `hub-agent-protocol`)
- [x] 1.2 Run `openspec validate add-agent-age-identity-distribution --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - Hub-side distribution
- [ ] 2.1 Add helper to copy `backup_age_identity/<key_name>` from Hub scope to Agent scope (re-encrypt per node)
- [ ] 2.2 Integrate distribution into Agent-executed restore orchestration (ensure key before restore starts)
- [ ] 2.3 Append audit operation events for distribution/usage (no secret payloads)
- [ ] 2.4 Commit backend changes (detailed message)

## 3. Hub→Agent secrets snapshot
- [ ] 3.1 Extend secrets snapshot payload to include distributed age identity entries
- [ ] 3.2 Persist age identity entries on Agent (encrypted-at-rest) using the existing managed snapshot mechanism
- [ ] 3.3 Commit hub/agent snapshot changes (detailed message)

## 4. Verification
- [ ] 4.1 Run `cargo test --workspace`
