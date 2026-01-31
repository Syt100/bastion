# Change: Surface Job Detail actions as a top toolbar

## Why
The Job Detail page currently hides job-level actions (edit/deploy/archive/delete) under the Settings tab.
This adds friction for common workflows (e.g. edit a job, deploy changes, archive a job) because users must navigate into a secondary tab to find controls.

## What Changes
- Add a Job Detail “actions toolbar” near the top of `/n/:nodeId/jobs/:jobId` so that common job-level actions are always visible.
- Keep destructive actions (archive/delete) confirmed via an explicit modal.
- Keep the Runs/Snapshots/Retention navigation as-is.
- Update the Job Detail Settings tab to no longer be the primary location for job-level actions.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/jobs/JobDetailShellView.vue`
  - `ui/src/views/jobs/JobDetailSettingsView.vue`
  - (optional) related i18n + unit tests

## Non-Goals
- Changing backend APIs or job schemas.
- Adding new job actions beyond what already exists in the UI (this is a UX/IA change).

