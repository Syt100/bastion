## 1. Spec
- [x] 1.1 Add `backend` spec delta for run/operation progress snapshots + filesystem `pre_scan`
- [x] 1.2 Add `web-ui` spec delta for filesystem `pre_scan` editor support
- [x] 1.3 Run `openspec validate add-backup-restore-progress --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Storage / Repo
- [x] 2.1 Add DB migration: `runs.progress_json` and `operations.progress_json`
- [x] 2.2 Add repo helpers to get/set progress snapshots

## 3. Engine / Backup / Targets
- [x] 3.1 Emit run progress snapshots during filesystem backup (scan/package/upload) with throttling
- [x] 3.2 Emit operation progress snapshots during restore/verify with throttling
- [x] 3.3 Add upload progress callbacks to targets (WebDAV + local_dir) so upload can report bytes done

## 4. Agent Execution
- [x] 4.1 Emit run progress snapshots from agent-run backups (filesystem upload/package)
- [x] 4.2 Emit operation progress snapshots from agent-run restore tasks

## 5. HTTP API
- [x] 5.1 Include `progress` in run/operation read responses where applicable

## 6. Web UI
- [x] 6.1 Add filesystem `pre_scan` toggle to the job editor (default ON) with help text
- [x] 6.2 Add/adjust unit tests for spec<->form mapping

## 7. Validation
- [x] 7.1 Run `cargo test --workspace`
- [x] 7.2 Run `pnpm -C ui test` (if present in repo)

## 8. Commits
- [x] 8.1 Commit implementation changes (detailed message with Modules/Tests)
