# Change: Add Job Schedule Timezone and UI Builder

## Why
Job scheduling is currently configured via a raw cron expression without an explicit timezone, which causes:
- User confusion about *when* a job will run (cron is implicitly interpreted in UTC today).
- Higher error rate for common schedules (daily/hourly/weekly) that could be expressed through a guided UI.
- Unclear behavior around DST transitions in timezones that observe DST.

## What Changes
- Backend:
  - Add a per-job `schedule_timezone` (IANA timezone, e.g. `Asia/Shanghai`), defaulting to the Hub timezone.
  - Interpret cron schedules in the configured timezone and evaluate due times consistently across Hub scheduling and agent offline scheduling.
  - Define DST semantics (gap=skip, fold=run once).
  - Tighten cron validation: accept 5-field cron; accept 6-field cron only when `seconds == 0` to avoid “valid but never triggers” schedules.
  - Expose Hub timezone via `/api/system` for UI defaults.
  - Include `schedule_timezone` in agent config snapshots.
- Web UI:
  - Replace the single cron text box with a schedule editor that supports:
    - Manual (empty)
    - Simple mode (daily/hourly/weekly/monthly/every N minutes)
    - Advanced cron expression mode
  - Add a timezone selector (default Hub timezone), and show the generated/effective cron.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - DB: `crates/bastion-storage/migrations/*`
  - Storage: `crates/bastion-storage/src/jobs_repo/*`
  - API: `crates/bastion-http/src/http/system`, `crates/bastion-http/src/http/jobs/*`, `crates/bastion-http/src/http/agents/snapshots.rs`
  - Scheduler: `crates/bastion-engine/src/scheduler/cron.rs`, `crates/bastion/src/agent_client/offline/scheduler/cron_loop.rs`
  - UI: `ui/src/components/jobs/editor/*`, `ui/src/stores/*`, `ui/src/i18n/locales/*`

## Compatibility / Non-Goals
- No requirement to preserve existing production behavior (project not launched yet), but behavior will be well-defined going forward.
- This change does not add new job types or modify backup/restore semantics.

