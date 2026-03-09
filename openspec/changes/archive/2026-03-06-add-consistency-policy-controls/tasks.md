## 1. Specification
- [x] Write spec deltas for `backup-jobs`, `control-plane`, `web-ui`, `notifications`
- [x] `openspec validate add-consistency-policy-controls --strict`

## 2. Implementation (in order)

### 2.1 Job spec + docgen
- [x] Extend job spec structs for filesystem and vaultwarden sources with policy fields
- [x] Update docgen reference checks

### 2.2 Hub execution path
- [x] Enforce policy after packaging, before upload (filesystem + vaultwarden)
- [x] Ensure report is persisted in summary for failed runs too

### 2.3 Agent execution path
- [x] Enforce the same policy in agent tasks (filesystem + vaultwarden)

### 2.4 Web UI job editor
- [x] Add editor controls (policy + threshold + upload flag) with tooltips
- [x] Validate inputs and ensure serialization into job spec
- [x] Add UI unit tests

### 2.5 Notifications
- [x] Include explicit messaging for `source_consistency` failures
- [x] Add template tests

## 3. Validation
- [x] Run `scripts/ci.sh`
