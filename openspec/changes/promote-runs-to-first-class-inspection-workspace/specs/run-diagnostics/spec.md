## ADDED Requirements

### Requirement: Run Detail Provides Structured Root-Cause Diagnostics
The system SHALL provide structured run diagnostics suitable for concise display before raw event payloads.

#### Scenario: Dedicated run view exposes normalized failure fields
- **WHEN** the client requests the dedicated run detail view model
- **THEN** the response SHALL include normalized diagnostic fields such as failure kind, failure stage, failure title, failure hint, and first-error location when known
- **AND** the UI SHALL be able to render a concise diagnosis from those fields without parsing raw event text

#### Scenario: Missing normalization falls back safely
- **GIVEN** a run does not have complete normalized diagnostics
- **WHEN** the detail view model is returned
- **THEN** the response SHALL still include raw or fallback diagnostic information sufficient for the UI to render a useful summary
- **AND** the UI SHALL preserve access to raw event details

#### Scenario: Structured diagnostics include root-cause locator
- **WHEN** the dedicated run detail view model is returned for a failed or degraded run
- **THEN** the diagnostics payload SHALL include a first-error or root-cause event locator when known
- **AND** that locator SHALL be usable by the event console without scanning the entire history client-side

### Requirement: Run Event Console Supports Server-Driven Filtering
The system SHALL provide a run-event contract that supports server-driven filtering and paging for the dedicated run workspace.

#### Scenario: Event query filters by text, level, and kind
- **WHEN** the client requests run events with text, level, or kind filters
- **THEN** the system SHALL return only events matching those criteria
- **AND** the response SHALL remain compatible with further pagination or cursor traversal

#### Scenario: Event history can be paged without fetching all rows
- **GIVEN** a run has a large event history
- **WHEN** the client requests the event console data incrementally
- **THEN** the system SHALL support cursor or equivalent pagination semantics
- **AND** the client SHALL NOT be required to fetch the complete event history before rendering the console

#### Scenario: Event windows remain sequence ordered
- **WHEN** the client receives an event-console response
- **THEN** the returned events SHALL be ordered by ascending event sequence within the current window
- **AND** the response SHALL expose enough metadata to determine whether older or newer matching events exist

#### Scenario: Filters apply before paging window is selected
- **WHEN** the client requests an event window with text, level, or kind filters
- **THEN** the server SHALL determine the matching slice before applying cursor/window limits
- **AND** the resulting window SHALL remain stable for that filter set instead of paging over the unfiltered event stream

### Requirement: Run Diagnostics Support First-Error Navigation
The dedicated diagnostics contract SHALL support locating the first meaningful failure quickly.

#### Scenario: Run view locates the first error event
- **GIVEN** a run contains one or more error events
- **WHEN** the client requests the run detail view or event-console metadata
- **THEN** the system SHALL expose enough information to navigate to the first relevant error event directly

#### Scenario: Event console can open anchored at the first error
- **GIVEN** first-error or root-cause location metadata is available
- **WHEN** the client requests the event console anchored to that location
- **THEN** the response SHALL return a window centered on or beginning from the requested anchor
- **AND** the UI SHALL not need to manually fetch preceding pages until it reaches the first error

### Requirement: Event Detail Prioritizes Diagnostics Before Raw Payloads
Event inspection within the run workspace SHALL present concise diagnostics before verbose payload sections.

#### Scenario: Event detail opens with concise diagnostic fields first
- **WHEN** an operator opens an event detail from the run workspace
- **THEN** the UI SHALL present localized or structured diagnostics before raw JSON or raw field dumps
- **AND** raw payload sections SHALL remain available through progressive disclosure

### Requirement: The Run Workspace Supports Live Event Refresh And Recovery
The dedicated run workspace SHALL support live or resumable event updates for active runs.

#### Scenario: Active run reconnects its event stream
- **GIVEN** a run is active and the live event connection is interrupted
- **WHEN** the operator remains on the run detail page
- **THEN** the workspace SHALL surface connection state
- **AND** it SHALL support reconnecting and continuing event inspection without losing the current run context

#### Scenario: Live refresh resumes from the last rendered sequence
- **GIVEN** the run workspace has already rendered an event window
- **WHEN** live refresh reconnects successfully
- **THEN** it SHALL resume from the last rendered sequence rather than restarting the entire event history
- **AND** it SHALL preserve the current filter context when the transport supports filtered live updates
