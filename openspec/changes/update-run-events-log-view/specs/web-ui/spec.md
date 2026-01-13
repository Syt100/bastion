## ADDED Requirements

### Requirement: Run Events Are Rendered As A Log List With Compact Rows
The Web UI SHALL render Run Events as a log list optimized for scanning and long-running tasks.

#### Scenario: Desktop row is single-line and scannable
- **WHEN** the viewport is `>= md`
- **THEN** each Run Event row shows time + level + kind + up to 2 summary chips + message (ellipsis) + Details

#### Scenario: Mobile row is compact and readable
- **WHEN** the viewport is `< md`
- **THEN** each Run Event row uses a compact two-line layout without excessive row height growth

### Requirement: Run Events Show Up To Two High-Signal Summary Chips
The Web UI SHALL display up to 2 summary chips per event derived from `event.fields` to help users quickly interpret outcomes (attempts, error kind, next retry time, durations, counts, etc.).

#### Scenario: Only two chips are rendered
- **GIVEN** an event has more than 2 eligible summary fields
- **WHEN** the Run Events list renders the row
- **THEN** at most 2 chips are shown

#### Scenario: Retry scheduling uses relative time
- **GIVEN** an event includes `next_attempt_at`
- **WHEN** the Run Events list renders the row
- **THEN** the value is shown in relative time (e.g., minutes from now)

### Requirement: Follow/Tail Behavior Matches Common Log Viewers
The Web UI SHALL support a “follow latest” mode by default and SHALL avoid fighting the user when they scroll up.

#### Scenario: Follow turns off when user scrolls away
- **GIVEN** follow mode is enabled
- **WHEN** the user scrolls away from the bottom of the list
- **THEN** follow mode is disabled

#### Scenario: New events are counted while follow is off
- **GIVEN** follow mode is disabled
- **WHEN** new events arrive
- **THEN** the UI shows an incrementing “new events” count
- **AND** the user can click “Latest” to jump to bottom and re-enable follow

### Requirement: WS Auto-Reconnect Is Enabled By Default
The Web UI SHALL automatically reconnect the Run Events websocket when disconnected, and SHALL provide a manual reconnect action.

#### Scenario: Auto reconnect attempts are visible
- **WHEN** the websocket disconnects unexpectedly
- **THEN** the UI shows a reconnecting state and the countdown to the next attempt

### Requirement: Details View Is Optimized For Desktop And Mobile
The Web UI SHALL provide an event details view for full message and fields JSON.

#### Scenario: Desktop uses a modal detail view
- **WHEN** the viewport is `>= md`
- **THEN** details open in a modal

#### Scenario: Mobile uses a half-screen bottom drawer
- **WHEN** the viewport is `< md`
- **THEN** details open in a bottom drawer (~70vh)

