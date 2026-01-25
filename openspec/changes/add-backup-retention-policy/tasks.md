## 1. Spec & Design
- [ ] Define retention config shape (`enabled`, `keep_last`, `keep_days`, safety limits)
- [ ] Define selection rules (keep set is union of keep-last and keep-days; exclude pinned)
- [ ] Validate this change with `openspec validate add-backup-retention-policy --strict`

## 2. Job Spec + Validation
- [ ] Extend `bastion_core::job_spec` to include `retention` (optional)
- [ ] Update spec validation rules (bounds, defaults, sane limits)
- [ ] Update job editor UI model to read/write retention config
- [ ] Add unit tests for spec parsing/validation

## 3. Global Defaults (Hub Settings)
- [ ] Add Hub runtime config fields for default retention
- [ ] Ensure new jobs inherit defaults, but allow per-job override
- [ ] Add tests for config persistence and defaulting behavior

## 4. Retention APIs
- [ ] `GET /api/jobs/:job_id/retention`
- [ ] `PUT /api/jobs/:job_id/retention`
- [ ] `POST /api/jobs/:job_id/retention/preview`
- [ ] `POST /api/jobs/:job_id/retention/apply`
- [ ] Add HTTP tests for auth + preview correctness

## 5. Engine: Retention Loop
- [ ] Add a retention loop that periodically:
  - lists candidate snapshots from `run_artifacts`
  - computes keep/delete sets
  - enqueues delete tasks (bounded per tick/day)
- [ ] Ensure pinned snapshots are excluded
- [ ] Add engine tests for selection correctness and safety limits

## 6. Web UI
- [ ] Add a "Retention" section to the job editor:
  - enable switch
  - keep_last, keep_days, safety limit
  - preview button + preview results viewer
  - apply-now button
- [ ] Add UI tests for retention editor and preview display

