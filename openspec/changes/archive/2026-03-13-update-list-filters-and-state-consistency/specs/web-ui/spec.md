## ADDED Requirements

### Requirement: List Filters SHALL Use Shared Field Presentation
List pages SHALL present filter labels and controls through shared field presentation primitives in both inline and stacked modes.

#### Scenario: Inline and stacked filter panels share visual structure
- **GIVEN** a page renders filters inline on desktop and stacked on constrained layouts
- **WHEN** the user opens or interacts with filters
- **THEN** filter label/control structure remains consistent between modes
- **AND** pages avoid duplicating handcrafted label/select/switch wrappers

### Requirement: List Pages SHALL Expose Unified Filter Summary Feedback
List pages SHALL expose result summary and active-filter chips through a shared summary row pattern.

#### Scenario: Filter effects are visible at a predictable location
- **GIVEN** one or more filters are active
- **WHEN** the list content renders
- **THEN** result count and active filter chips are shown in a consistent summary row
- **AND** clearing filters uses a shared interaction pattern

### Requirement: List State Feedback SHALL Reuse Shared Presenter
Loading, base-empty, and filtered-empty states SHALL be rendered through a shared state presenter component.

#### Scenario: Empty-state semantics stay consistent across pages
- **GIVEN** Jobs and Agents list pages can be loading, empty, or filtered-empty
- **WHEN** there are no rows to render
- **THEN** the page uses a shared state presenter
- **AND** action affordances (create/clear) are surfaced with consistent structure
