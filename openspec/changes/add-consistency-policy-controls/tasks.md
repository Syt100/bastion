## 1. Specification
- [ ] Write spec deltas for `backup-jobs`, `control-plane`, `web-ui`, `notifications`
- [ ] `openspec validate add-consistency-policy-controls --strict`

## 2. Implementation (in order)

### 2.1 Job spec + docgen
- [ ] Extend job spec structs for filesystem and vaultwarden sources with policy fields
- [ ] Update docgen reference checks

### 2.2 Hub execution path
- [ ] Enforce policy after packaging, before upload (filesystem + vaultwarden)
- [ ] Ensure report is persisted in summary for failed runs too

### 2.3 Agent execution path
- [ ] Enforce the same policy in agent tasks (filesystem + vaultwarden)

### 2.4 Web UI job editor
- [ ] Add editor controls (policy + threshold + upload flag) with tooltips
- [ ] Validate inputs and ensure serialization into job spec
- [ ] Add UI unit tests

### 2.5 Notifications
- [ ] Include explicit messaging for `source_consistency` failures
- [ ] Add template tests

## 3. Validation
- [ ] Run `scripts/ci.sh`

