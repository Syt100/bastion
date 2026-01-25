## MODIFIED Requirements

### Requirement: Run Detail Overview/Progress Are Compact
The Run Detail page SHALL present Overview and Progress panels with compact spacing and minimal empty areas.

#### Scenario: Overview card does not stretch unnecessarily
- **GIVEN** the Run Detail page is rendered in a desktop grid layout
- **THEN** the Overview card does not stretch to match the height of adjacent cards
- **AND** large blank areas are avoided

### Requirement: Final Transfer Speed Is Visible After Completion
The Progress panel SHALL show a meaningful transfer speed after completion.

#### Scenario: Completed run shows speed even when `rate_bps` is missing
- **GIVEN** a completed run/operation with transfer totals available
- **AND** the progress snapshot has no `rate_bps`
- **THEN** the UI computes and displays a final transfer speed
- **AND** it does not display "-" for speed when the data is computable
