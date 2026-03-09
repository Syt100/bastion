## ADDED Requirements

### Requirement: Run Events WebSocket Uses `after_seq`
The Web UI SHALL connect the run events WebSocket with `after_seq` equal to the last known event sequence to avoid receiving duplicate catch-up events.

#### Scenario: No duplicate catch-up events after initial load
- **WHEN** the user opens the Run Events viewer and the UI has already loaded events up to sequence N
- **THEN** the WebSocket connection includes `after_seq = N`
- **AND** the UI does not process duplicated events for sequences `<= N`

### Requirement: Run Events Viewer Supports Large Event Counts
The Run Events viewer SHALL remain responsive for runs with large numbers of events.

#### Scenario: Viewer remains responsive with many events
- **WHEN** a run produces a large number of events
- **THEN** the UI uses an efficient rendering strategy (e.g., virtualization or fixed-height rows) to avoid rendering all events at once

### Requirement: Event Details Are Shown On Demand
The Run Events viewer SHALL avoid rendering large JSON payloads inline by default and SHALL provide an on-demand way to inspect event details (such as `fields`).

#### Scenario: Event fields are viewed in a details UI
- **WHEN** an event contains structured `fields`
- **THEN** the user can open a details view to inspect the JSON

### Requirement: Follow Mode Preserves User Scroll Position
The Run Events viewer SHALL support a follow mode (auto-scroll to latest events) that can be disabled to preserve the user’s scroll position while reading historical output.

#### Scenario: Follow mode disabled preserves scroll
- **WHEN** follow mode is disabled and new events arrive
- **THEN** the UI does not automatically scroll to the bottom

