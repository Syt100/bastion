# Change: Run Events follow auto-resume when returning to bottom

## Why
Today, when “follow” is auto-disabled after the user scrolls away from the bottom, returning to the bottom still requires an extra explicit action (click “Latest” or toggle the follow switch).

Most log viewers treat “being at the bottom” as equivalent to “following”, but still respect explicit user intent when follow was manually turned off.

## What Changes
- Track why follow was disabled:
  - **Auto**: disabled because the user scrolled away while follow was on.
  - **Manual**: disabled explicitly via the follow switch.
- If follow is **auto-disabled**, scrolling back to the bottom automatically re-enables follow (and clears the “new events” count).
- If follow is **manually disabled**, scrolling back to the bottom does **not** auto-enable follow.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/components/jobs/RunEventsModal.vue` and its unit tests.

## Non-Goals
- Adding new UI controls or copy changes.
- Changing websocket reconnect behavior.

