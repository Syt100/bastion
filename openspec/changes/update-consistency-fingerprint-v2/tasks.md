## 1. Specification
- [x] Write spec deltas for `sources`, `backup-jobs`, `control-plane`, `web-ui`
- [x] `openspec validate update-consistency-fingerprint-v2 --strict`

## 2. Implementation (in order)

### 2.1 Data model: v2 report types
- [x] Add v2 structs (fingerprint + report + sample) and helpers in `bastion-backup`
- [x] Implement Windows `file_id` best-effort extraction

### 2.2 Filesystem builder integration
- [x] Update filesystem archive writer to capture `before`, `after_handle`, `after_path`
- [x] Update raw_tree writer to capture `before`, `after_handle` (copy handle), `after_path`

### 2.3 Vaultwarden builder integration
- [x] Update vaultwarden tar writer to record v2 fingerprints where applicable

### 2.4 Reporting and UI
- [x] Emit v2 report in run events and `summary_json`
- [x] Update UI parsing + rendering for v2 report
- [x] Update API-derived totals (runs list) if schema changes require it

### 2.5 Tests
- [x] Add/adjust unit tests for v2 report serialization and fingerprint extraction
- [x] Add platform-specific regression tests where feasible (unix/windows)

## 3. Validation
- [x] Run `scripts/ci.sh`
