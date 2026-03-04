## ADDED Requirements

### Requirement: Picker Lists SHALL Ignore Stale Responses
Picker list surfaces SHALL protect refresh/load-more data state from stale asynchronous responses.

#### Scenario: Older request resolves after a newer filter/path request
- **GIVEN** a picker list has triggered request A and then request B with newer state
- **WHEN** request A resolves after request B
- **THEN** request A result SHALL NOT override the currently displayed rows/cursor state
- **AND** loading indicators SHALL settle according to the latest active request

### Requirement: Picker Loaded-Row Selection Semantics SHALL Be Shared
Picker surfaces that support multi-row selection SHALL share one loaded-row selection model for select-all, invert, and shift-range behaviors.

#### Scenario: Shared loaded-row selection is reused across picker modals
- **GIVEN** two picker modals expose loaded-row selection controls
- **WHEN** the user triggers select-all/invert/shift-range actions
- **THEN** both modals SHALL derive the next selected set from the same shared selection logic
- **AND** selection from non-loaded pages/paths SHALL remain preserved where applicable

### Requirement: Jobs Workspace Filters SHALL Use Shared Filter Modeling
Jobs workspace list filtering SHALL use the shared list filter model for active-count/chip generation and clear behavior.

#### Scenario: Jobs filter chips/count use shared model
- **GIVEN** the user applies search and/or select-based filters in Jobs workspace
- **WHEN** filter chips and active filter count are rendered
- **THEN** chips/count SHALL be derived through the shared filter model utility
- **AND** clear-all SHALL reset to Jobs-defined defaults without page-local duplicated chip/count logic

### Requirement: Picker/List Query Serialization SHALL Be Shared
Picker/list request query parameter serialization for common filters SHALL be provided through shared helpers.

#### Scenario: Shared serialization keeps query semantics stable
- **GIVEN** picker-like list requests with search, kind, dotfiles, type sort, size range, and sort options
- **WHEN** requests are serialized to query parameters
- **THEN** shared helpers SHALL produce consistent parameter keys and value normalization
- **AND** migrated surfaces SHALL preserve existing backend contract semantics

### Requirement: Debounce And Abort Guards SHALL Be Shared Utilities
List views with debounced refresh and abort-aware error handling SHALL reuse shared utility helpers.

#### Scenario: Debounced refresh + abort guard reuse
- **GIVEN** list views that debounce refresh or swallow abort cancellation errors
- **WHEN** those views are implemented
- **THEN** debounce scheduling and abort-error detection SHALL be provided by shared utility helpers
- **AND** equivalent views SHALL avoid reimplementing ad-hoc timer/abort detection logic
