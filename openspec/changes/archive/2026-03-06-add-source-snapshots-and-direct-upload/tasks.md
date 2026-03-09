## 1. Specification
- [x] Write spec deltas for `sources`, `backup-jobs`, `hub-agent`, `targets-local-dir`, `targets-webdav`, `control-plane`, `web-ui`
- [x] `openspec validate add-source-snapshots-and-direct-upload --strict`

## 2. Implementation (phased, in order)

### 2.1 Snapshot framework + Btrfs provider (Phase 1)
- [x] Add snapshot abstractions and run events
- [x] Implement Linux Btrfs provider behind an allowlist/enable flag
- [x] Add job spec config for snapshot mode/provider (filesystem source)
- [x] Use snapshot path for packaging when snapshot is ready
- [x] Add tests (probe + required mode failure)

### 2.2 LocalDir raw_tree reduced staging (Phase 1)
- [x] Add “direct data path” mode for raw_tree + local_dir targets
- [x] Ensure completion marker semantics remain unchanged
- [x] Add tests for no-dup staging and correct target layout

### 2.3 Web UI (Phase 1)
- [x] Job editor: snapshot mode/provider controls (filesystem)
- [x] Run detail: snapshot status + provider info
- [x] Add i18n and UI tests

### 2.4 WebDAV raw_tree direct upload (Phase 2)
- [x] Design + implement raw_tree direct uploader to WebDAV (streaming + hashing)
- [x] Add failure cleanup strategy
- [x] Add tests and performance guardrails (concurrency/limits)

## 3. Validation
- [x] Run `scripts/ci.sh`
