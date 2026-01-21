## Context

We persist only the latest progress snapshot per run/operation, and the Web UI currently renders it as a single text line.

For filesystem backups using raw_tree_v1, the upload code currently discovers data files during upload and grows the upload total while the upload is in progress. Combined with throttling, this often results in snapshots where DONE and TOTAL are nearly equal and both grow together, which prevents users from seeing a stable total or a meaningful percentage.

## Goals / Non-Goals

Goals:
- Provide stable, user-meaningful totals across stage transitions (SOURCE totals and TRANSFER totals).
- Provide a clear Progress panel UI on Run Detail (desktop + mobile) with:
  - overall progress bar (percent when possible)
  - stage breakdown with per-stage explanation
  - key stats: source vs transfer, speed, ETA, last update

Non-Goals:
- Historical progress timeline charts.
- Perfect time-to-completion prediction.

## Decisions

### Decision: Keep the existing progress snapshot envelope; extend via DETAIL
- Keep ProgressSnapshotV1 (stage, done, total, rate_bps, eta_seconds) as the top-level envelope.
- Add structured backup-specific fields under detail.backup to carry stable totals and stage breakdown data.

Proposed detail.backup fields (JSON):
- source_total: { files, dirs, bytes } (optional)
- packaging_done: { files, dirs, bytes } (optional; present when known)
- transfer_total_bytes: number (optional)
- transfer_done_bytes: number (optional)
- overall_pct: number (0-100, optional)
- overall_explain: string (optional; describes how percent was computed)

Notes:
- The UI uses detail.backup.source_total to show stable dataset totals even after the scan stage.
- The UI uses transfer_* fields to show upload progress percent.

### Decision: Make raw_tree_v1 upload totals stable
- When the run transitions into upload, compute a stable transfer total bytes:
  - meta bytes: entries index + manifest + complete marker (+ any parts)
  - raw_tree data bytes: measured during packaging (raw-tree build stats)
- During upload progress emission, always publish the stable transfer total bytes.

This avoids the "DONE ~= TOTAL and both grow" effect and enables an accurate upload percentage.

### Decision: Compute an overall percentage in the Web UI
- A single overall percent is helpful, but not all totals are always known at the beginning of a run.
- We compute overall percent using stage weights, and fall back to indeterminate when the current stage does not have a computable percent.

Default stage weights:
- scan: 5%
- packaging: 45%
- upload: 50%

Stage percent inputs:
- scan: percent only if scan emits totals (best-effort)
- packaging: done.bytes / total.bytes when totals exist (pre_scan enabled)
- upload: transfer_done_bytes / transfer_total_bytes when total exists

We also show per-stage progress bars so users can see what is currently happening.

## Risks / Trade-offs

- Adding structured fields to detail increases complexity, but keeps the top-level snapshot stable.
- Overall percent is an estimate; stage weights may not match all workloads. We mitigate this by also showing per-stage breakdown and clear labels.

## Migration Plan

- No DB migration required (progress snapshots are stored as JSON).
- UI should tolerate missing detail.backup fields and fall back to the previous stage/done/total rendering.

## Open Questions

- Whether to also expose upload file counts (artifacts uploaded) in addition to byte counts.
