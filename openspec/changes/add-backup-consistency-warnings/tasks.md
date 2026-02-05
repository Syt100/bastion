## 1. Specification
- [x] Write spec deltas for `sources`, `backup-jobs`, `control-plane`, `web-ui`
- [x] `openspec validate add-backup-consistency-warnings --strict`

## 2. Implementation (in order)

### 2.1 Single-read hashing + consistency detection (builders)
- [x] Add shared consistency types/utilities (fingerprint + report + capped samples)
- [x] Refactor filesystem `archive_v1` tar writer to hash while writing (single read) and collect consistency warnings
- [x] Add consistency warnings to filesystem `raw_tree_v1` copy path
- [x] Refactor Vaultwarden tar writer to hash while writing (single read) and collect consistency warnings
- [x] Add/adjust builder return types to expose consistency report to callers
- [x] Add regression tests in `crates/bastion-backup` for:
  - single-read hash correctness (entries index hash matches archived bytes)
  - change detection when a file mutates during backup

### 2.2 Run reporting (events + summary)
- [x] Hub execution path: emit `source_consistency` run event when changes detected (filesystem + vaultwarden)
- [x] Agent execution path: emit `source_consistency` run event when changes detected (filesystem + vaultwarden)
- [x] Include structured consistency report in run `summary_json` (filesystem + vaultwarden)

### 2.3 UI: run detail warning
- [x] Extend `ui/src/lib/run_summary.ts` to parse consistency report
- [x] Show consistency warning tag in run detail summary card
- [x] Add i18n strings (`zh-CN`, `en-US`)
- [x] Add UI unit tests for the new summary parsing / render conditions

### 2.4 UI + API: job runs list warning
- [x] Extend `GET /api/jobs/:id/runs` response to include `consistency_changed_total`
- [x] Update UI `RunListItem` type and job history table to show a warning tag when `consistency_changed_total > 0`
- [x] Add UI unit tests for job runs list warning tag

## 3. Validation
- [x] Run `scripts/ci.sh`
- [x] Ensure warning sample sizes are capped and no large payloads are stored in SQLite
