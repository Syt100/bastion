# Change: Run Events log viewer UX (chips, reconnect, mobile details drawer)

## Why
The Run Events viewer is used as a real-time “log tail” for runs, but today it is harder than necessary to:
- Quickly scan for key signals (attempts, error kind, next retry time, durations, counts).
- Understand connection state and recover automatically from transient WS disconnects.
- Use on mobile without the details view feeling cramped.

## What Changes
- Render Run Events as a log list with a consistent row layout:
  - Desktop: single-line row with time + level + kind + up to 2 “field chips” + message (ellipsis) + Details.
  - Mobile: compact two-line row (line 1: time/level/message; line 2: kind + chips).
- Surface up to 2 high-signal “field chips” per event derived from `event.fields` (e.g., `error_kind`, `attempt(s)`, `next_attempt_at` (relative), `duration_ms`, `errors_total/warnings_total`, etc.).
- Improve follow/tail behavior:
  - Auto-disable “follow” when user scrolls away from the bottom.
  - Show a “new events” counter and a one-click “Latest” action to jump back and re-enable follow.
- Improve WS connection status:
  - Show clearer states (connecting / live / disconnected / reconnecting / error).
  - Enable auto-reconnect by default with exponential backoff and a manual reconnect button.
- Improve details UX:
  - Keep a Details button and allow row click to open details.
  - Desktop: modal details.
  - Mobile: bottom drawer (half-screen).

## Impact
- Affected specs: `web-ui`
- Affected code: Run Events UI (`ui/src/components/jobs/RunEventsModal.vue`) and related i18n/tests.

## Non-Goals
- Pagination/virtualization: typical run event counts are only a few hundred.
- URL-synced filters/search for this view.

