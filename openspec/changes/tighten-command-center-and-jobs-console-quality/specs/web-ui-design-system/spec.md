## ADDED Requirements

### Requirement: Operational Surfaces Use Professional Control-Plane Copy
Primary operational pages SHALL use professional, concise, and evidence-first copy.

#### Scenario: Supporting text avoids slogan framing
- **WHEN** a primary operational page renders headings, subtitles, helper text, or quiet-state copy
- **THEN** the wording SHALL describe status, evidence, consequence, or follow-up action directly
- **AND** the UI SHALL NOT rely on promotional, slogan-like, or conversational filler text

### Requirement: Primary Actions And Attention States Have Clear Readability
The design system SHALL keep actionable states and primary controls visually stronger than neutral metadata while preserving readable contrast.

#### Scenario: Primary action remains visually unambiguous
- **WHEN** a page presents a main action such as `Create job`, `Run now`, `Open run`, or `Continue`
- **THEN** that action SHALL remain visually stronger than adjacent neutral utilities such as refresh, mode toggles, or passive metadata
- **AND** nearby controls SHALL not dilute the page's main action hierarchy

#### Scenario: Attention styling remains legible on quiet backgrounds
- **GIVEN** operational pages use soft panel backgrounds or low-chrome shells
- **WHEN** caution, failure, degraded, or blocking states are rendered
- **THEN** those states SHALL preserve readable foreground/background contrast and visual distinction from neutral metadata
- **AND** the page SHALL NOT rely on near-equal low-contrast treatments for both risk and neutral information
