## 1. Spec & Design
- [x] Define retention config shape (`enabled`, `keep_last`, `keep_days`, safety limits)
- [x] Define selection rules (keep set is union of keep-last and keep-days; exclude pinned)
- [x] Validate this change with `openspec validate add-backup-retention-policy --strict`

## 2. Job Spec + Validation
- [x] Extend `bastion_core::job_spec` to include `retention` (optional)
- [x] Update spec validation rules (bounds, defaults, sane limits)
- [x] Update job editor UI model to read/write retention config
- [x] Add unit tests for spec parsing/validation

## 3. Global Defaults (Hub Settings)
- [x] Add Hub runtime config fields for default retention
- [x] Ensure new jobs inherit defaults, but allow per-job override
- [x] Add tests for config persistence and defaulting behavior

## 4. Retention APIs
- [x] `GET /api/jobs/:job_id/retention`
- [x] `PUT /api/jobs/:job_id/retention`
- [x] `POST /api/jobs/:job_id/retention/preview`
- [x] `POST /api/jobs/:job_id/retention/apply`
- [x] Add HTTP tests for auth + preview correctness

## 5. Engine: Retention Loop
- [x] Add a retention loop that periodically:
  - lists candidate snapshots from `run_artifacts`
  - computes keep/delete sets
  - enqueues delete tasks (bounded per tick/day)
- [x] Ensure pinned snapshots are excluded
- [x] Add engine tests for selection correctness and safety limits

## 6. Web UI
- [x] Add a "Retention" section to the job editor:
  - enable switch
  - keep_last, keep_days, safety limit
  - preview button + preview results viewer
  - apply-now button
- [x] Add UI tests for retention editor and preview display
