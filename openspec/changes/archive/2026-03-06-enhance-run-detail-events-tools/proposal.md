# Change: Run Detail Events Filtering, Search, and Export Tools

## Why
The events stream is useful for diagnosis but becomes inefficient to navigate without filtering, search, and quick actions (jump to error, copy/export).

## What Changes
- Add an events toolbar with:
  - Search by message/kind.
  - Filter by level (info/warn/error) and kind.
  - Quick navigation actions (jump to first error/warn, jump to latest).
- Improve event inspection ergonomics:
  - Message is single-line truncated in the list, with hover/click to view full details.
  - Add copy/export helpers (copy single event JSON; export filtered events JSON).
- Improve connection/empty states:
  - Clear status display for websocket state and user-facing actions (reconnect).

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/views/RunDetailView.vue` (or extracted components), i18n strings
- No backend changes
