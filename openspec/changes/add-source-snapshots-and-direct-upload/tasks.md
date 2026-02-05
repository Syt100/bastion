## 1. Specification
- [ ] Write spec deltas for `sources`, `backup-jobs`, `hub-agent`, `targets-local-dir`, `targets-webdav`, `control-plane`, `web-ui`
- [ ] `openspec validate add-source-snapshots-and-direct-upload --strict`

## 2. Implementation (phased, in order)

### 2.1 Snapshot framework + Btrfs provider (Phase 1)
- [ ] Add snapshot abstractions and run events
- [ ] Implement Linux Btrfs provider behind an allowlist/enable flag
- [ ] Add job spec config for snapshot mode/provider (filesystem source)
- [ ] Use snapshot path for packaging when snapshot is ready
- [ ] Add tests (probe + required mode failure)

### 2.2 LocalDir raw_tree reduced staging (Phase 1)
- [ ] Add “direct data path” mode for raw_tree + local_dir targets
- [ ] Ensure completion marker semantics remain unchanged
- [ ] Add tests for no-dup staging and correct target layout

### 2.3 Web UI (Phase 1)
- [ ] Job editor: snapshot mode/provider controls (filesystem)
- [ ] Run detail: snapshot status + provider info
- [ ] Add i18n and UI tests

### 2.4 Optional: WebDAV raw_tree direct upload (Phase 2)
- [ ] Design + implement raw_tree direct uploader to WebDAV (streaming + hashing)
- [ ] Add failure cleanup strategy
- [ ] Add tests and performance guardrails (concurrency/limits)

## 3. Validation
- [ ] Run `scripts/ci.sh`

