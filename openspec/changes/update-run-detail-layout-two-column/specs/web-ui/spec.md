## MODIFIED Requirements

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: Desktop uses a two-column first screen layout
- **GIVEN** a run detail is loaded on a desktop viewport
- **THEN** the page presents a readable Summary + Progress panel alongside the Details area
- **AND** the Details area presents Events / Operations / Summary as tabs

#### Scenario: Mobile uses a single-column first screen layout
- **GIVEN** a run detail is loaded on a mobile viewport
- **THEN** the page presents Summary + Progress before the Details tabs
- **AND** the page does not require horizontal scrolling

## ADDED Requirements

### Requirement: Run Detail Summary/Progress Remains Accessible on Desktop
The Run Detail page SHALL keep the Summary + Progress panel accessible while the user browses the Details area on desktop.

#### Scenario: Summary/Progress remains visible while browsing details
- **GIVEN** the user scrolls within the Details area on a desktop viewport
- **THEN** the Summary + Progress panel remains visible without requiring the user to scroll back to the top

### Requirement: Run Detail Presents a Cohesive Header and Action Area
The Run Detail page SHALL present run status, target information, and primary actions as a cohesive header/action area.

#### Scenario: Header uses localized labels and consistent actions
- **GIVEN** a run is loaded
- **THEN** the run status is displayed using localized text
- **AND** target information is displayed using product-friendly labels
- **AND** Restore/Verify actions are disabled when the run is not eligible
