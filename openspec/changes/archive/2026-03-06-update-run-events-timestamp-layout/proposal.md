# Change: Run Events timestamp layout (no-wrap + mobile compact)

## Why
The Run Events list currently allows the timestamp to wrap (e.g., date + time split into two lines). Because rows are rendered with a fixed height (virtual list), wrapping causes the two lines to appear cramped and collide with row borders.

## What Changes
- Ensure the timestamp column never wraps and has comfortable vertical spacing within the fixed row height.
- Make the timestamp display responsive:
  - Desktop (`>= md`): show a compact date+time format suitable for scanning.
  - Mobile (`< md`): show a concise time-only format (`HH:mm`).
- Keep full timestamp information accessible via the event details view (and/or hover title on desktop).

## Impact
- Affected specs: `web-ui`
- Affected code: Run Events UI (`ui/src/components/jobs/RunEventsModal.vue`)

