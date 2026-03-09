## MODIFIED Requirements

### Requirement: Run Detail Summary Card
The Run Detail page SHALL render Overview and Progress inside a single summary card to avoid large blank areas caused by uneven card heights.

#### Scenario: Desktop default shows full summary
- **GIVEN** the Run Detail page is rendered on a desktop viewport
- **THEN** Overview and Progress information is shown in a single summary card
- **AND** the summary is expanded by default (no additional user action required)

#### Scenario: No large blank area under Overview
- **GIVEN** the Progress section is taller than the Overview section
- **THEN** the UI avoids a large visible blank area under Overview by using a single-card layout

### Requirement: Restore Final Speed in Operation Details
The Operation details modal SHALL display a meaningful speed after a restore operation completes.

#### Scenario: Completed restore shows average speed when live rate is missing
- **GIVEN** an operation of kind `restore` with `ended_at` set
- **AND** a progress snapshot exists with `done.bytes` > 0
- **AND** the latest progress snapshot has no `rate_bps`
- **THEN** the modal computes and displays a final average speed
