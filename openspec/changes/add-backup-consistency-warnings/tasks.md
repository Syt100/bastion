## 1. Specification
- [ ] Write spec deltas for `sources`, `backup-jobs`, `control-plane`, `web-ui`
- [ ] `openspec validate add-backup-consistency-warnings --strict`

## 2. Implementation (in order)

### 2.1 Single-read hashing + consistency detection (builders)
- [ ] Add shared consistency types/utilities (fingerprint + report + capped samples)
- [ ] Refactor filesystem `archive_v1` tar writer to hash while writing (single read) and collect consistency warnings
- [ ] Add consistency warnings to filesystem `raw_tree_v1` copy path
- [ ] Refactor Vaultwarden tar writer to hash while writing (single read) and collect consistency warnings
- [ ] Add/adjust builder return types to expose consistency report to callers
- [ ] Add regression tests in `crates/bastion-backup` for:
  - single-read hash correctness (entries index hash matches archived bytes)
  - change detection when a file mutates during backup

### 2.2 Run reporting (events + summary)
- [ ] Hub execution path: emit `source_consistency` run event when changes detected (filesystem + vaultwarden)
- [ ] Agent execution path: emit `source_consistency` run event when changes detected (filesystem + vaultwarden)
- [ ] Include structured consistency report in run `summary_json` (filesystem + vaultwarden)

### 2.3 UI: run detail warning
- [ ] Extend `ui/src/lib/run_summary.ts` to parse consistency report
- [ ] Show consistency warning tag in run detail summary card
- [ ] Add i18n strings (`zh-CN`, `en-US`)
- [ ] Add UI unit tests for the new summary parsing / render conditions

### 2.4 UI + API: job runs list warning
- [ ] Extend `GET /api/jobs/:id/runs` response to include `consistency_changed_total`
- [ ] Update UI `RunListItem` type and job history table to show a warning tag when `consistency_changed_total > 0`
- [ ] Add UI unit tests for job runs list warning tag

## 3. Validation
- [ ] Run `scripts/ci.sh`
- [ ] Ensure warning sample sizes are capped and no large payloads are stored in SQLite

