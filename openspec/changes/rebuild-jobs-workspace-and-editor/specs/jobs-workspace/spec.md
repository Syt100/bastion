## ADDED Requirements

### Requirement: Jobs Uses A Three-Pane Desktop Workspace
The desktop Jobs surface SHALL use a workspace layout that keeps job discovery, selected job context, and supporting operational detail visible at the same time.

#### Scenario: Desktop layout keeps list context while inspecting a job
- **WHEN** an operator opens the Jobs surface on a desktop viewport
- **THEN** the page SHALL provide a list/discovery region and a selected-job detail region in the same workspace
- **AND** the workspace SHALL provide a third supporting region or equivalent secondary pane for recent run, health, or related operational context

#### Scenario: Narrow desktop widths collapse gracefully
- **GIVEN** the desktop viewport is too narrow for three simultaneous panes
- **WHEN** the Jobs workspace renders
- **THEN** the third pane SHALL collapse into a secondary tab, drawer, or equivalent subordinate pattern
- **AND** the primary list/detail workflow SHALL remain usable without horizontal overflow

#### Scenario: Mobile uses dedicated list and detail pages
- **WHEN** the Jobs surface renders on a mobile viewport
- **THEN** the workspace SHALL switch to dedicated list and detail routes instead of preserving the desktop split layout mechanically
- **AND** the operator SHALL still be able to reach filtering, selection, and primary detail actions without hidden desktop-only affordances

### Requirement: Jobs Uses A Canonical Stable Route Family
The Jobs workspace SHALL expose stable top-level routes for collection, detail, and authoring flows.

#### Scenario: Stable route identifies the job object instead of node-prefixed path
- **WHEN** the operator opens job detail or job-edit flows
- **THEN** the route identity SHALL use a canonical top-level job path
- **AND** any list scope or return context SHALL be carried separately from the stable object path

### Requirement: Jobs Filters Are Persistent And Discoverable
The Jobs workspace SHALL keep primary filters visible and understandable without relying on hidden hover or small affordances.

#### Scenario: Desktop filters stay visible
- **WHEN** an operator views the desktop Jobs workspace
- **THEN** primary filters such as search, status, schedule, archived state, sort order, and scope-related filters SHALL be visible without opening a secondary popover

#### Scenario: Mobile filters remain fully available
- **WHEN** an operator views the Jobs workspace on mobile
- **THEN** the page SHALL expose all primary filters through a dedicated drawer, sheet, or equivalent mobile filter surface
- **AND** the active filter state SHALL remain visible in summary form after the filter surface closes

#### Scenario: Back-navigation restores list context
- **GIVEN** the operator opens a job detail page from a filtered Jobs collection
- **WHEN** they return to the collection
- **THEN** the collection SHALL restore the previously active scope, filters, sort order, and pagination context
- **AND** the user SHALL NOT need to rebuild that slice manually

### Requirement: Jobs Supports Reusable Saved Views
The Jobs workspace SHALL support reusable saved views for common operator slices.

#### Scenario: Operator reuses a named filtered job list
- **GIVEN** an operator saves a filtered job list definition
- **WHEN** they return to the Jobs workspace later
- **THEN** they SHALL be able to reapply the saved view directly
- **AND** the resulting list SHALL restore the saved filter/sort semantics

#### Scenario: Saved view captures scope and archive visibility
- **WHEN** the operator saves a Jobs view
- **THEN** the saved definition SHALL include the collection scope, archive visibility, and sort/filter state required to recreate that slice
- **AND** applying the saved view SHALL update the collection in one step instead of replaying field changes individually

### Requirement: Job Rows Expose Operational Summary And Direct Actions
The Jobs list SHALL render operationally meaningful row summaries and directly discoverable primary actions.

#### Scenario: Job row shows recent operational summary
- **WHEN** a job row is rendered
- **THEN** the row SHALL show at least job identity, effective scope or node context, recent run outcome, and timing signals relevant to operation
- **AND** the row SHALL make risk or degraded state visually apparent without opening detail first

#### Scenario: Job row actions are visible on touch and pointer workflows
- **WHEN** a job row is visible
- **THEN** primary actions such as open detail, run now, and more actions SHALL be discoverable without hover-only reveal
- **AND** action boundaries SHALL remain separate from row-selection or row-navigation behavior

### Requirement: Job Detail Uses A Workspace-Oriented Summary Model
The system SHALL provide a workspace-oriented job summary model rather than requiring the UI to assemble detail context from many unrelated calls.

#### Scenario: Job workspace view model includes planning and health context
- **WHEN** the UI requests the selected job workspace view
- **THEN** the response SHALL include job identity, recent success/failure signals, next schedule, target-related summary, and readiness or warning metadata needed for the workspace
- **AND** the UI SHALL be able to render the primary detail summary from that single view model

#### Scenario: Workspace detail response exposes operator capabilities
- **WHEN** the UI requests the selected job workspace view
- **THEN** the response SHALL expose capability flags or equivalent metadata for run-now, edit, archive, delete, and related actions
- **AND** the UI SHALL NOT need to infer action availability from loosely related fields

### Requirement: Mobile Jobs Uses Dedicated List And Detail Flows
The mobile Jobs experience SHALL separate job list browsing from job detail inspection.

#### Scenario: Mobile job detail is a dedicated page
- **WHEN** a mobile operator opens a job
- **THEN** the UI SHALL navigate to a dedicated job detail page
- **AND** the page SHALL keep top-level actions and section navigation usable without requiring the desktop split layout

#### Scenario: Mobile job actions remain explicit
- **WHEN** a mobile operator opens a job detail page
- **THEN** primary actions such as run now, edit, and refresh SHALL remain explicitly discoverable
- **AND** the mobile action area SHALL NOT depend on icon-only controls as the sole affordance for common job operations
