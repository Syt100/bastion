## ADDED Requirements

### Requirement: Mobile Step Navigation Uses Compact Current-Step Progress
Mobile job authoring SHALL use a compact progress pattern instead of a fully expanded horizontal stage strip.

#### Scenario: Mobile step navigation stays readable
- **WHEN** a mobile operator opens the job editor
- **THEN** the page SHALL display the current step identity and overall progress in a readable compact form
- **AND** the editor SHALL NOT require seven compressed step labels to remain simultaneously visible in the first viewport

#### Scenario: Step jumping remains available without crowded chrome
- **WHEN** a mobile operator needs to inspect or jump to another step
- **THEN** the editor SHALL expose a secondary step-navigation affordance such as a sheet, menu, or expandable progress control
- **AND** the operator SHALL NOT need to horizontally scroll a crushed step strip to navigate

### Requirement: Mobile Summaries Collapse Before Primary Inputs
On mobile, configuration and risk summaries SHALL collapse into subordinate containers before the active form loses clarity.

#### Scenario: First viewport prioritizes active fields and primary actions
- **WHEN** the job editor renders on a mobile viewport
- **THEN** the first viewport SHALL prioritize the current step inputs plus the primary action area
- **AND** full configuration or risk summaries SHALL be collapsed, deferred, or peeked instead of occupying primary space by default

#### Scenario: Blocking warnings remain near the action area
- **GIVEN** the current step or draft state has blocking validation or high-severity risk signals
- **WHEN** the mobile editor renders with collapsed summaries
- **THEN** the editor SHALL repeat the blocking warning near the primary action area
- **AND** the operator SHALL NOT be forced to open a secondary summary container just to discover why progression is blocked
