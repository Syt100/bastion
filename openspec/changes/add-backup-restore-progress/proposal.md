# Change: Add backup/restore progress snapshots and filesystem pre-scan

## Why
Backup and restore work can take a long time, but the UI currently only shows coarse events, which makes it hard to estimate how much work remains and whether the system is making progress.

## What Changes
- Persist a **latest progress snapshot** for:
  - backup runs (`runs.progress_json`)
  - long-running operations (`operations.progress_json`, e.g. restore/verify)
- Emit periodic progress updates (throttled) during backup packaging/upload and restore/verify execution.
- Add a filesystem job option `source.pre_scan` (default: `true`) to optionally pre-scan the filesystem to compute totals and ETA.
- Expose progress snapshots via existing authenticated APIs (run/operation reads).

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Storage: migrations + repo helpers (`crates/bastion-storage/*`)
  - Engine/backup: progress instrumentation (`crates/bastion-engine/*`, `crates/bastion-backup/*`)
  - Agent execution: progress events for agent-run jobs / restores (`crates/bastion/src/agent_client/*`)
  - Targets: per-artifact upload progress hooks (`crates/bastion-targets/*`)
  - UI: filesystem job editor option + progress rendering hooks (used by Run Detail page change)

## Non-Goals
- Precise compression-level “on-disk” size prediction during packaging.
- Historical progress timelines (only “latest snapshot” is persisted; events remain available separately).

