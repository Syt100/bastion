## ADDED Requirements

### Requirement: Control-Console Pages Use A Dominant Panel Hierarchy
Primary control-console pages SHALL use one dominant work surface with subordinate rails or inset panels instead of many peer-level elevated cards.

#### Scenario: Landing page uses one primary work surface
- **WHEN** a top-level operational page such as the Command Center renders
- **THEN** the page SHALL present one dominant surface hierarchy for the main content
- **AND** secondary summaries or controls SHALL be visually subordinate through rails, insets, or equivalent non-competing treatments

#### Scenario: Secondary rails collapse before the primary workspace loses clarity
- **WHEN** the viewport becomes too narrow to support multiple desktop columns cleanly
- **THEN** secondary rails or tertiary panels SHALL collapse into tabs, drawers, or other subordinate containers first
- **AND** the primary work surface SHALL remain visually dominant and readable

### Requirement: Shell Chrome Uses Restrained Navigation Surfaces
The shell SHALL render navigation chrome with restrained surfaces that support orientation without competing visually with the active workspace.

#### Scenario: Navigation chrome remains subordinate to the page
- **WHEN** the shell renders sidebar, topbar, or mobile drawer chrome
- **THEN** those surfaces SHALL remain visually subordinate to the primary page workspace
- **AND** the shell SHALL NOT rely on repeated elevated cards to separate every navigation section

#### Scenario: Mobile shell reuses the same hierarchy without desktop framing
- **WHEN** the shell renders on a mobile viewport
- **THEN** the drawer and sticky top bar SHALL preserve the same information hierarchy as desktop navigation
- **AND** the mobile shell SHALL NOT recreate desktop sidebars as stacked card sections above the primary page

### Requirement: Attention States Have Stronger Visual Priority Than Neutral Metadata
The design system SHALL reserve the strongest visual emphasis for actionable status and attention states rather than applying equal weight to all counters or metadata blocks.

#### Scenario: Failure state outranks neutral count state
- **GIVEN** a page contains both a failure summary and neutral informational totals
- **WHEN** the page is rendered
- **THEN** the failure summary SHALL have stronger visual emphasis than the neutral totals
- **AND** the page SHALL NOT present both with equivalent hierarchy

### Requirement: Mobile Layouts Follow Task-First Content Order
Mobile layouts for primary operational pages SHALL order content by immediate task relevance instead of preserving desktop section order mechanically.

#### Scenario: Mobile page surfaces actionable content first
- **WHEN** an operator opens a primary operational page on mobile
- **THEN** the first viewport SHALL prioritize the sections most likely to drive immediate action
- **AND** secondary summaries or low-signal overview content SHALL be pushed below the actionable content

### Requirement: Route Transitions Preserve Orientation Cues
Primary-shell transitions SHALL preserve enough visible context for operators to understand where they are after route normalization or cross-surface navigation.

#### Scenario: Normalized migration alias still shows the target surface clearly
- **GIVEN** the operator lands on a canonical route via a temporary migration alias
- **WHEN** the target page renders
- **THEN** the active top-level surface and any relevant contextual scope or return chip SHALL be visible without opening extra menus
- **AND** the normalized page SHALL not appear detached from the shell hierarchy
