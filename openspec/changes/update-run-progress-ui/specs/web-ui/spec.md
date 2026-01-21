## ADDED Requirements

### Requirement: Run Detail Shows A Progress Panel
The Web UI SHALL render a dedicated Progress panel on the node-scoped Run Detail page, replacing the single-line progress text.

#### Scenario: User can read overall progress at a glance
- **GIVEN** a user opens a running backup run on Run Detail
- **WHEN** the UI loads the latest progress snapshot
- **THEN** the Progress panel shows an overall progress bar and key stats without requiring the user to parse a single long text line

### Requirement: Progress Panel Shows Stage Breakdown With Help
The Progress panel SHALL show the backup stages (Scan, Packaging, Upload) with per-stage progress and a help entrypoint ("?") explaining each stage.

#### Scenario: User opens packaging stage help
- **GIVEN** a backup run is in the packaging stage
- **WHEN** the user clicks the "?" help entrypoint for Packaging
- **THEN** the UI shows a short explanation of what Packaging is doing for the selected backup format

### Requirement: Progress Panel Is Mobile-Friendly
The Progress panel SHALL adapt to mobile screens using a stacked layout and collapsible sections while preserving readability.

#### Scenario: Progress panel remains usable on mobile
- **GIVEN** a user opens Run Detail on a small screen
- **WHEN** the Progress panel is displayed
- **THEN** key progress information remains visible without requiring horizontal scrolling
