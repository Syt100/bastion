# Change: Improve backup progress semantics and Run Detail progress UI

## Why
The current Run Detail page renders progress as a single text line, which makes it hard to understand backup status at a glance. In particular for raw_tree_v1, the upload stage currently reports a dynamically-growing total, so users may never see a stable total size or a meaningful percentage.

## What Changes
- Backend: enrich backup progress snapshots so they expose both SOURCE totals and TRANSFER totals (where applicable), and carry stable totals across stage transitions.
- Backend: for raw_tree_v1 uploads, provide a stable upload total (bytes) so percent/ETA are meaningful.
- Web UI: replace the single-line progress text on Run Detail with a dedicated Progress panel:
  - Overall progress bar (percent when possible, indeterminate otherwise)
  - Stage stepper (Scan -> Packaging -> Upload) with per-stage progress and ? help
  - Key stats (source vs transfer) with bytes, files/dirs, rate, ETA, and last update
  - Mobile-friendly layout (stacked + collapsible sections)

## Impact
- Affected specs: backend, web-ui
- Affected code:
  - Backup/engine progress emitters
  - Run Detail UI components

## Non-Goals
- Historical progress timeline charts (we still persist only the latest snapshot).
