## 1. Specification
- [ ] Write spec deltas for `sources`, `backup-jobs`, `control-plane`, `web-ui`
- [ ] `openspec validate update-consistency-fingerprint-v2 --strict`

## 2. Implementation (in order)

### 2.1 Data model: v2 report types
- [ ] Add v2 structs (fingerprint + report + sample) and helpers in `bastion-backup`
- [ ] Implement Windows `file_id` best-effort extraction

### 2.2 Filesystem builder integration
- [ ] Update filesystem archive writer to capture `before`, `after_handle`, `after_path`
- [ ] Update raw_tree writer to capture `before`, `after_handle` (copy handle), `after_path`

### 2.3 Vaultwarden builder integration
- [ ] Update vaultwarden tar writer to record v2 fingerprints where applicable

### 2.4 Reporting and UI
- [ ] Emit v2 report in run events and `summary_json`
- [ ] Update UI parsing + rendering for v2 report
- [ ] Update API-derived totals (runs list) if schema changes require it

### 2.5 Tests
- [ ] Add/adjust unit tests for v2 report serialization and fingerprint extraction
- [ ] Add platform-specific regression tests where feasible (unix/windows)

## 3. Validation
- [ ] Run `scripts/ci.sh`

