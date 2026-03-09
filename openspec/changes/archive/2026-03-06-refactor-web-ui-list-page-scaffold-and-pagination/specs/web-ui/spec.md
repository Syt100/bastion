## ADDED Requirements

### Requirement: List Pages Use A Shared Structural Scaffold
The Web UI SHALL provide a shared list-page scaffold that standardizes the structure of selection tools, filter/action toolbar, list content region, and pagination footer across list-oriented pages.

#### Scenario: Shared scaffold regions are rendered consistently
- **GIVEN** a list-oriented page that opts into the shared scaffold
- **WHEN** the page renders in desktop or mobile layout
- **THEN** selection tools (if any), toolbar controls, content area, and pagination footer are rendered in stable, predictable regions
- **AND** page-specific business widgets are injected through explicit slots/regions instead of custom page wrappers

### Requirement: List Pagination Interaction Is Consistent Across Pages
List-oriented pages SHALL expose consistent pagination behavior and controls, including page number, page-size selection, total count visibility, and disabled/loading states.

#### Scenario: Notifications queue follows the same pagination pattern as Jobs and Agents
- **GIVEN** the user is viewing Notifications Queue, Jobs list, or Agents list
- **WHEN** the user navigates pages or changes page size
- **THEN** each page presents equivalent pagination controls and semantics
- **AND** pagination disabled/loading behavior is consistent across these pages

#### Scenario: Filter changes reset pagination predictably
- **GIVEN** a user is on page N of a list
- **WHEN** the user updates search/filter criteria
- **THEN** the list resets to the first page before fetching updated results
- **AND** the behavior is consistent for all migrated list pages

### Requirement: Empty States Support Context-Aware Surface Variants
The Web UI SHALL provide empty-state rendering variants that avoid nested card surfaces when an empty state is rendered inside an existing card/list container.

#### Scenario: Empty state inside list card does not create nested card noise
- **GIVEN** a list page content area already uses a card/surface container
- **WHEN** the list enters loading-empty or no-data state
- **THEN** the empty state uses a non-card variant (`plain` or `inset`)
- **AND** the page does not render a card-within-card empty-state pattern

### Requirement: List Visual Hierarchy Avoids Redundant Decorative Layers
List pages SHALL avoid stacking redundant decorative wrappers around toolbars and empty/content regions, preserving a single dominant surface hierarchy per section.

#### Scenario: Toolbar and empty/list content do not create stacked decorative wrappers
- **GIVEN** a migrated list page with a primary card/surface
- **WHEN** toolbars and empty states are rendered
- **THEN** the page uses at most one dominant elevated surface for the list section
- **AND** additional wrappers are neutral/inset rather than repeated elevated cards
