## ADDED Requirements

### Requirement: Page Surfaces SHALL Expose Consistent Visual Hierarchy
The web UI SHALL provide consistent hierarchy levels for page titles, section titles, and metadata text across shared layout components.

#### Scenario: Shared headers and list shells render consistent hierarchy
- **GIVEN** pages render `PageHeader` and list scaffolds
- **WHEN** the user navigates between Jobs and Agents list pages
- **THEN** title, subtitle, and metadata text use consistent visual hierarchy classes
- **AND** hierarchy levels do not depend on per-page ad-hoc utility combinations

### Requirement: Shared Surfaces SHALL Reduce Decorative Noise
The web UI SHALL tune shared chrome tokens so backgrounds, borders, and shadows prioritize data readability over decoration.

#### Scenario: Content remains dominant across themes
- **GIVEN** the user views a list-heavy page in any supported theme
- **WHEN** cards, toolbars, and list containers are displayed
- **THEN** surface styling uses reduced-noise defaults (subtler background intensity and chrome)
- **AND** primary content contrast remains clear in light and dark modes

### Requirement: List Scaffolds SHALL Use Unified Spacing Rhythm
Shared list layout components SHALL enforce consistent vertical spacing for toolbar, content, and pagination zones.

#### Scenario: Two list pages share spacing cadence
- **GIVEN** two pages use `ListPageScaffold`
- **WHEN** both render toolbar, content area, and pagination
- **THEN** the vertical spacing cadence is consistent
- **AND** page-local overrides are no longer required for basic rhythm

### Requirement: Dense List Metadata SHALL Follow Shared Emphasis Rules
List rows and data-table secondary metadata SHALL use shared low-emphasis styles to preserve scanability in dense datasets.

#### Scenario: Row metadata remains readable without competing with primary labels
- **GIVEN** a list row contains primary text and secondary metadata
- **WHEN** the row is rendered in list or table mode
- **THEN** primary text remains visually dominant
- **AND** secondary metadata uses shared reduced-emphasis styles
