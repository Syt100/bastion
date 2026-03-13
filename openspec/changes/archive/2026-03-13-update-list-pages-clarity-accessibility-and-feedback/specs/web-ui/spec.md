## ADDED Requirements

### Requirement: Jobs Results Summary Uses Explicit Visible And Filtered Semantics
The Jobs workspace SHALL display list metrics that clearly distinguish currently visible items from filtered total matches.

#### Scenario: Jobs list shows visible count and filtered total
- **GIVEN** the Jobs workspace list is loaded
- **WHEN** the current page has M rows and the filtered dataset has N total rows
- **THEN** the UI shows both M and N in the results summary
- **AND** the summary no longer renders an ambiguous duplicated total expression

### Requirement: Jobs Mobile Filter Visibility Matches Desktop Parity
The Jobs workspace SHALL surface active filter conditions in mobile list mode using the same filter-chip source of truth as desktop layouts.

#### Scenario: Mobile layout shows and clears active filters via chips
- **GIVEN** the user applies search or filter controls in Jobs mobile list mode
- **WHEN** the list content renders
- **THEN** active filter chips are visible above list content
- **AND** chip close / clear actions reset the same underlying filter model used in desktop layouts

### Requirement: Jobs Row Activation Semantics Are Explicit And Accessible
Jobs list rows SHALL separate row-main activation from nested action controls using explicit interactive elements with keyboard-friendly semantics.

#### Scenario: Row-main activation and row-action activation do not conflict
- **GIVEN** a Jobs list row with row-main activation and nested action controls
- **WHEN** the user triggers a row action control
- **THEN** only the action executes
- **AND** row-main navigation/select behavior does not fire

#### Scenario: Row-main activation supports keyboard interaction
- **GIVEN** a Jobs list row in list mode
- **WHEN** keyboard users focus and activate the row-main trigger
- **THEN** the same row-main behavior executes as pointer activation

### Requirement: Notifications Queue Provides Explicit Empty-State Variants
Notifications Queue SHALL provide explicit empty-state guidance for loading-empty, base empty, and filtered-no-results states.

#### Scenario: Queue shows context-aware empty state messaging
- **GIVEN** Notifications Queue has no rows to display
- **WHEN** state is loading-empty, no queued entries, or filter-no-match
- **THEN** the page shows state-appropriate empty-state title/description/actions
- **AND** users can recover quickly (for example, via clear filters or refresh)

### Requirement: List Pagination Summaries Include Visible Range And Total
List pages that use shared pagination SHALL expose a consistent visible-range summary with total count.

#### Scenario: Jobs, Agents, and Notifications show range summary consistently
- **GIVEN** a paginated list page among Jobs, Agents, or Notifications Queue
- **WHEN** pagination footer is rendered
- **THEN** the summary includes visible start/end indices and total count
- **AND** formatting is consistent across these pages

### Requirement: Agents Mobile Cards Prioritize Primary Information
Agents mobile cards SHALL prioritize primary scan fields and move secondary metadata into progressive disclosure.

#### Scenario: Secondary metadata is collapsed by default on mobile cards
- **GIVEN** the user views Agents on mobile viewport
- **WHEN** agent cards render
- **THEN** primary identity/status/actions remain directly visible
- **AND** less-critical metadata is accessible through an explicit expand/collapse affordance

### Requirement: Key Async Row Actions Provide In-Flight Feedback
List row actions for high-frequency operations SHALL provide immediate in-flight feedback and duplicate-submit prevention.

#### Scenario: Triggering row action sets local busy state
- **GIVEN** a user triggers a key row action (for example, "Run now" or notification retry/cancel)
- **WHEN** the async request is in flight
- **THEN** the corresponding control shows loading/busy feedback
- **AND** repeated triggers for the same row action are temporarily disabled until completion

### Requirement: Search-Driven List Refresh Uses A Shared Debounce Cadence
Search-driven list refresh behavior SHALL use a shared debounce cadence across list pages that support text query search.

#### Scenario: Jobs and Agents search refresh cadence is consistent
- **GIVEN** the user types in Jobs or Agents search input
- **WHEN** query text changes rapidly
- **THEN** refresh requests are deferred by the same debounce interval before fetch
- **AND** the effective cadence is consistent across both pages
