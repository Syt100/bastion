# Change: Add source snapshots and direct upload pipelines

## Why
Two recurring operational pain points remain:

1) **Source consistency**: without snapshots/quiescing, filesystem backups can be internally inconsistent. We already warn best-effort, but operators want a real point-in-time view where supported.

2) **Local staging**: some pipelines (notably `raw_tree_v1`) stage large amounts of data on local disk before upload. This increases disk pressure and slows down runs.

We want to:
- Use native snapshot mechanisms when available (Btrfs/ZFS/VSS, etc.) to produce a stable read view.
- Reduce or eliminate local staging by writing to the target as early as possible, while preserving the atomic “complete marker written last” semantics.

## What Changes

### Source snapshots
- Introduce a snapshot subsystem with a `SnapshotProvider` abstraction.
- Add job/source configuration:
  - `snapshot_mode: off|auto|required`
  - `snapshot_provider` (optional override)
- Emit run events:
  - `snapshot_started`, `snapshot_ready`, `snapshot_unavailable`, `snapshot_cleanup_failed`
- Use the snapshot path as the read root for packaging when present.

### Direct upload / reduced staging
- `archive_v1`: continue rolling part upload (already exists). Consider optional “write parts directly to local_dir target” when target is local.
- `raw_tree_v1`:
  - Add a direct-to-target mode for `local_dir` targets to avoid duplicating the full data tree under local staging.
  - Provide a phased plan for WebDAV direct upload (optional, higher complexity).

## Impact
- Affected specs: `sources`, `backup-jobs`, `hub-agent`, `targets-local-dir`, `targets-webdav`, `control-plane`, `web-ui`
- Affected code (expected):
  - `crates/bastion-backup/src/backup/filesystem/*`
  - `crates/bastion-engine/src/scheduler/worker/execute/*`
  - `crates/bastion/src/agent_client/tasks/*`
  - `crates/bastion-targets/src/local_dir.rs`
  - `crates/bastion-targets/src/webdav.rs`
  - `ui/src/components/jobs/editor/*` (snapshot config)
  - `ui/src/components/runs/*` (snapshot status)

## Compatibility / Non-Goals
- Snapshot support is best-effort by default (`auto`) and must have a clean fallback path.
- We do not attempt to implement every provider on day 1; we prioritize one robust provider (Linux Btrfs) and provide scaffolding for others.
- We do not weaken atomicity: a run is considered present only once the `complete` marker exists in the target.

