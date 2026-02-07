## 1. Specification
- [x] Write spec deltas for `targets-webdav`, `backup-jobs`, `control-plane`, `web-ui`
- [x] `openspec validate update-webdav-direct-upload-safety-and-alert-digest --strict`

## 2. Implementation (in order)

### 2.1 Job/pipeline explicit config for WebDAV raw-tree direct upload
- [ ] Add `pipeline.webdav.raw_tree_direct` settings to job spec types
- [ ] Add validation for supported targets/formats and policy conflicts
- [ ] Update engine: remove auto-enable; enable only when explicitly configured
- [ ] Web UI: expose direct upload mode + limits in filesystem job editor

### 2.2 WebDAV request limits + concurrent uploads
- [ ] Add request limiter (concurrency + qps) to `WebdavClient` and apply to PUT/HEAD/MKCOL
- [ ] Add backpressure handling for 429/503 (+ Retry-After) in upload paths
- [ ] Make staged raw-tree `data/` uploads concurrent (bounded)
- [ ] Make raw-tree direct upload concurrent (bounded pipeline) while preserving index ordering
- [ ] Add regression tests (limits, completion marker last, concurrency ceiling)

### 2.3 Alert digest + runs list de-noising
- [ ] Extend runs list endpoint (`GET /api/jobs/:id/runs`) to return high-signal digest fields
- [ ] Update Web UI runs list to show capped high-signal badges only (and keep detail evidence)
- [ ] Add backend + UI tests for digest and de-noising rules

## 3. Validation
- [ ] Run `scripts/ci.sh`
