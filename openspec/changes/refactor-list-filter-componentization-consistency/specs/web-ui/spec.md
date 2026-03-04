## ADDED Requirements

### Requirement: List Filters Use Shared Modeling For Active State And Chips
List pages that expose filter controls SHALL use a shared local filter-model utility for active-state derivation, clear behavior, and active-filter chip generation.

#### Scenario: Shared filter model drives clear and chip state
- **GIVEN** a list page with search and/or select-based filters
- **WHEN** filters are changed
- **THEN** active-filter count/chips are derived from the shared filter model
- **AND** clear-all resets the same model back to page-defined defaults

### Requirement: List Filter Controls Reuse Shared Field Wrappers
List pages that render select-based toolbar filters SHALL reuse shared list filter field wrappers for consistent sizing and toolbar presentation.

#### Scenario: Filter select controls render with consistent width and layout
- **GIVEN** list pages with one or more select filters in toolbar regions
- **WHEN** those filters are rendered on desktop and mobile breakpoints
- **THEN** shared field wrappers provide consistent container sizing and select control layout
- **AND** pages avoid repeating ad-hoc width utility blocks for equivalent filter fields

### Requirement: Active Filter Visibility Is Consistent Across Core List Pages
Agents, Notifications Queue, Maintenance Cleanup, and Job Snapshots SHALL display active-filter chips with a consistent clear affordance.

#### Scenario: Migrated list pages show active chips and support clear-all
- **GIVEN** the user applies one or more filters on Agents, Notifications Queue, Maintenance Cleanup, or Job Snapshots
- **WHEN** list content is displayed
- **THEN** an active-filter chip row appears using the shared component path
- **AND** users can clear all filters from the same row-level clear action

### Requirement: Picker Modals Reuse Shared Filter Modeling For Active Chips
Run entries and path picker modals SHALL reuse the shared local filter model for active-filter count/chips and clear behavior.

#### Scenario: Picker modals keep filter-count/chip behavior via shared model
- **GIVEN** the user applies filters in RunEntries picker or PathPicker modal
- **WHEN** the filters toolbar and active-chip row render
- **THEN** active filter count/chips are derived through shared filter-model utilities
- **AND** clear-all and per-chip close behaviors remain functional without page-local duplicated chip/count logic
