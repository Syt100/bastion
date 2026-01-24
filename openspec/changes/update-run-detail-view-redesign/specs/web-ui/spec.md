## MODIFIED Requirements

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: Run Detail uses a cohesive first screen layout
- **GIVEN** a run detail is loaded
- **THEN** the first screen presents a readable summary of the run (status + key facts)
- **AND** the run progress is visible in the first screen on both desktop and mobile

## ADDED Requirements

### Requirement: Run Detail Consolidates Secondary Sections
The Run Detail page SHALL consolidate Events, Operations, and Summary into a single Details area.

#### Scenario: Desktop avoids long scrolling with tabs
- **GIVEN** the user is on a desktop viewport
- **THEN** the page presents Events / Operations / Summary as tabs
- **AND** empty sections do not render large placeholder tables/cards

#### Scenario: Mobile presents the same Details tabs
- **GIVEN** the user is on a mobile viewport
- **THEN** the page presents the same Events / Operations / Summary tabs in a mobile-friendly layout

### Requirement: Run Summary Hides Empty Blocks
The Run Detail page SHALL avoid rendering empty summary blocks.

#### Scenario: Summary only renders detail blocks when present
- **GIVEN** a run has a summary payload
- **WHEN** optional summary fields are absent
- **THEN** the page does not render empty placeholder panels for those fields
