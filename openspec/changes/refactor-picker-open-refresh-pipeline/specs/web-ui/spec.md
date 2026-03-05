## ADDED Requirements

### Requirement: Picker Session Opening SHALL Stage Transition and Initial Refresh
The web UI SHALL stage picker session startup so modal visibility/enter transition and first list refresh are sequenced to reduce frame contention during dialog open.

#### Scenario: Open transition is not blocked by immediate heavy refresh work
- **GIVEN** a user opens a directory picker dialog
- **WHEN** the session starts
- **THEN** the dialog is shown before heavy list refresh work begins
- **AND** initial data refresh still occurs automatically within the same open session

### Requirement: Picker Table Height Measurement SHALL Avoid Redundant Open-Frame Work
The web UI SHALL measure picker table body height with a stable lifecycle that avoids unnecessary repeated open-frame measurements.

#### Scenario: Open lifecycle performs stable measurement setup
- **GIVEN** a picker dialog opens
- **WHEN** table-body max height is initialized
- **THEN** the measurement lifecycle performs only the required initial measurement/setup steps
- **AND** subsequent re-measurements are driven by meaningful size changes instead of redundant chained frame callbacks

### Requirement: Picker Open/Refresh Performance Guards SHALL Include Unit Tests
The web UI SHALL include unit tests that assert picker open sequencing and measurement lifecycle behavior to prevent regressions.

#### Scenario: Unit tests detect sequencing regressions
- **GIVEN** picker session and table-height composables
- **WHEN** unit tests run in CI
- **THEN** tests verify open sequencing, refresh trigger timing, and measurement lifecycle expectations
- **AND** regressions that reintroduce open-time contention fail tests
