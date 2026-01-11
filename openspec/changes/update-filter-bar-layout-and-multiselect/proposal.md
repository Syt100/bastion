# Change: Update filter bars (desktop layout + multi-select)

## Why
Several list pages use a filter bar where each condition can take a full row on desktop, making the page unnecessarily tall.
Also, enum-like filters (e.g. status/channel/target) are currently single-select, which slows down operational workflows that often require viewing multiple states at once.

## What Changes
- Make filter bars compact on desktop (horizontal row with wrapping), while keeping mobile-friendly full-width controls.
- Add multi-select support for low-cardinality enum filters:
  - Incomplete run cleanup: `status`, `target_type`
  - Notifications queue: `status`, `channel`
- Extend backend list endpoints to accept multiple values for these filters via repeated query params.

## Impact
- Affected specs: `web-ui`, `backend`
- Affected code:
  - `ui/` (maintenance cleanup view, notifications queue view, stores)
  - `crates/bastion-http/` (query parsing/validation)
  - `crates/bastion-storage/` (list/count queries)

## Compatibility / Non-Goals
- Existing clients that send a single filter value continue to work.
- No new filter dimensions are introduced in this change.
