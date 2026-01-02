# Change: Update Notifications Management (Settings Submenu, Channels/Destinations/Templates/Queue, Job Overrides)

## Why
Current notification configuration is embedded in a single Settings page and lacks:
- A scalable information architecture for adding more settings and notification capabilities.
- Global/channel-level enable switches and per-destination enable/disable controls.
- A user-visible notification queue/records view with pagination and actions (retry/cancel).
- A synchronous “test notification” capability per destination.
- Per-job notification routing (choose channel + destinations per job).

These gaps make it hard to operate and troubleshoot notification delivery at scale and to extend the product later.

## What Changes
- Settings page becomes a small “Settings shell” with an internal submenu (route-based).
- Add `Settings / Notifications` area with subpages:
  - Channels (global + per-channel enable switches)
  - Destinations (enable/disable, CRUD via secrets, test send)
  - Templates (global templates only)
  - Queue (paged list of notifications + retry-now/cancel)
- Add per-job notification configuration:
  - Default: inherit global
  - Custom: select channel type (WeCom/SMTP) then select destinations (multi-select)
  - Disabled destinations can be selected with inline warnings
- Define clear semantics:
  - Turning off global/channel cancels queued notifications in-scope
  - Deleting a destination cancels queued notifications for that destination
  - “Retry now” resets attempts and schedules immediate send

## Impact
- Affected specs: `notifications`, `web-ui`, `backup-jobs`
- Affected code: backend DB migrations/repos, engine enqueue/worker, HTTP APIs, and Web UI routes/pages/forms

