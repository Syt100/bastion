## ADDED Requirements

### Requirement: Run Events Timestamp Is Non-Wrapping In Fixed Rows
The Web UI SHALL render the Run Events timestamp column as a single line (no wrapping) and SHALL provide sufficient vertical spacing so the timestamp does not visually collide with row borders.

#### Scenario: Timestamp does not wrap and stays readable
- **WHEN** the Run Events viewer displays events in a fixed-height virtual list row
- **THEN** the timestamp text remains on one line
- **AND** the timestamp is vertically comfortable (not touching borders)

### Requirement: Responsive Run Events Timestamp Format
The Web UI SHALL display a responsive timestamp format for Run Events:
- On desktop viewports (`>= md`): show a compact date+time format suitable for scanning.
- On mobile viewports (`< md`): show a concise time-only format (`HH:mm`).

#### Scenario: Desktop shows compact date+time
- **WHEN** the viewport is `>= md`
- **THEN** each Run Event row shows a compact date+time timestamp

#### Scenario: Mobile shows time-only
- **WHEN** the viewport is `< md`
- **THEN** each Run Event row shows a time-only timestamp in `HH:mm` format

### Requirement: Full Timestamp Remains Accessible
The Web UI SHALL ensure the full timestamp information remains accessible even when the list uses a compact format.

#### Scenario: Full timestamp can be viewed
- **WHEN** a user opens the Run Event details view
- **THEN** the full timestamp is visible in the details

