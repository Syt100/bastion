## ADDED Requirements

### Requirement: Core List Pages Reuse Shared Route-Filter Hydration
Core list pages SHALL reuse shared route-query parsing helpers for hydrating filter state from URL query values.

#### Scenario: Route query values are parsed with shared helper semantics
- **GIVEN** a list page reads filter values from `route.query`
- **WHEN** query values are strings, arrays, comma-separated strings, or invalid values
- **THEN** page filter state is hydrated via shared parsing helpers
- **AND** unknown values are ignored without breaking defaults

### Requirement: Server-Paginated List Pages Reuse Shared Pagination Behavior
Server-paginated list pages SHALL reuse shared pagination component behavior and shared page-size options to avoid per-page interaction drift.

#### Scenario: List pages expose consistent pagination controls
- **GIVEN** two or more server-paginated list pages
- **WHEN** users change page or page size
- **THEN** each page uses the same pagination component interaction model
- **AND** page-size option defaults are sourced from shared constants

### Requirement: Picker Open/Reset Lifecycle Uses Shared Model
Picker modals with large open/reset state blocks SHALL reuse a shared picker lifecycle/reset model.

#### Scenario: Picker opens with clean deterministic state
- **GIVEN** a picker modal is opened repeatedly with different contexts
- **WHEN** the picker initializes state for a new session
- **THEN** reset logic is executed through shared lifecycle helpers
- **AND** stale local UI state from prior sessions is cleared consistently

### Requirement: Per-Item Busy State Uses Shared Infrastructure
Views that track request-in-flight status by entity id SHALL reuse shared busy-state composables.

#### Scenario: Busy flags are managed through shared id helpers
- **GIVEN** a list view supports row-level operations with loading states
- **WHEN** an operation starts or finishes for a specific row id
- **THEN** busy state is updated via shared id-based helpers
- **AND** local pages avoid duplicating map-clone/delete boilerplate

### Requirement: Store List Query Serialization Reuses Shared Builders
Store list APIs SHALL reuse shared query serialization helpers for common list/filter/pagination parameter patterns.

#### Scenario: List stores serialize common parameters consistently
- **GIVEN** stores build `URLSearchParams` for paginated/filterable list endpoints
- **WHEN** filters and pagination parameters are applied
- **THEN** shared serializer helpers construct common query keys and values
- **AND** existing endpoint-specific parameter names remain unchanged

### Requirement: Jobs Workspace Row Rendering Is Componentized
Jobs workspace SHALL split duplicated row/table rendering into shared subcomponents while preserving current actions and selection behavior.

#### Scenario: Desktop/mobile row actions remain behaviorally equivalent after extraction
- **GIVEN** Jobs workspace renders list/table rows across desktop and mobile layouts
- **WHEN** users select rows, open details, run now, or open overflow actions
- **THEN** extracted row-rendering components preserve the same action semantics
- **AND** duplicated per-layout row markup is reduced through reusable components
