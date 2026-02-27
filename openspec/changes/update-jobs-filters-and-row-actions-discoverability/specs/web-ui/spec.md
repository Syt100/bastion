## ADDED Requirements

### Requirement: Jobs Filters Use A Single Source Of Truth
The Jobs workspace SHALL use one canonical filter state model for search, archived toggle, latest-run status, schedule mode, and sort order across all layout modes.

#### Scenario: Split/list/mobile layouts share the same filter values
- **GIVEN** the user has set Jobs filters in one layout mode
- **WHEN** the layout mode changes (split, full list, or mobile list)
- **THEN** the same filter values remain active
- **AND** the fetched results reflect the same criteria without mode-specific divergence

### Requirement: Jobs Filter Controls Render Consistently Across Containers
The Jobs workspace SHALL reuse the same filter control definitions across inline, popover, and drawer containers.

#### Scenario: Filter controls keep parity across container types
- **GIVEN** Jobs filters are shown in inline (desktop), popover (split), or drawer (mobile) container
- **WHEN** the user opens the filter controls
- **THEN** the available controls, option sets, and clear behavior are equivalent
- **AND** active-filter count/chips are derived from the same filter model

### Requirement: Primary Job Row Actions Are Discoverable Without Hover
The Jobs list SHALL expose primary row actions in a persistently visible way, without requiring hover-only reveal.

#### Scenario: User can trigger primary row actions immediately
- **GIVEN** the user views the Jobs list on pointer, touch, or keyboard workflow
- **WHEN** a row is visible
- **THEN** at least the primary action set (for example, Run Now and More) is directly visible
- **AND** the user does not need hover to discover these primary actions

### Requirement: Row Action Interaction Boundaries Are Explicit
Jobs row navigation and row action buttons SHALL have non-conflicting interaction boundaries.

#### Scenario: Action click does not trigger row navigation
- **GIVEN** a user activates a job row action button
- **WHEN** the action is triggered
- **THEN** only that action executes
- **AND** the row does not simultaneously open job detail navigation

#### Scenario: Row click still opens job detail
- **GIVEN** a user clicks the non-action area of a job row
- **WHEN** row click behavior runs
- **THEN** the Jobs workspace opens the selected job detail as before
