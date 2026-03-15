## ADDED Requirements

### Requirement: Command Center Uses Professional Operator-Facing Copy
The `Command Center` SHALL use concise operational wording instead of slogan-like or promotional framing.

#### Scenario: Hero copy states what the page shows
- **WHEN** the authenticated operator opens the `Command Center`
- **THEN** the primary heading and supporting copy SHALL describe the page in terms of risks, activity, readiness, or follow-up actions
- **AND** the page SHALL NOT rely on abstract slogan phrasing to explain its purpose

#### Scenario: Healthy and quiet states remain evidence-based
- **GIVEN** the selected scope has no current failures or urgent follow-up items
- **WHEN** the page renders healthy or empty-state sections
- **THEN** the copy SHALL explain the quiet state in operational terms
- **AND** it SHALL avoid sounding celebratory, promotional, or vague

### Requirement: Command Center First Screen Prioritizes Actionable Sections
The first visible portion of `Command Center` SHALL keep actionable status and recovery context above neutral metadata.

#### Scenario: First screen includes an actionable or readiness-led section
- **WHEN** the page renders on a common desktop viewport
- **THEN** the top section SHALL remain compact enough that at least one attention, readiness, or critical-activity section is visible without relying on long downward scrolling
- **AND** scope/timestamp echoes SHALL remain visually subordinate

#### Scenario: Neutral counters do not outrank caution state
- **GIVEN** the page contains both summary counters and degraded readiness or attention items
- **WHEN** the first screen is rendered
- **THEN** degraded readiness and follow-up items SHALL have stronger visual priority than neutral counts
- **AND** neutral counters SHALL NOT read as the dominant outcome of the page
