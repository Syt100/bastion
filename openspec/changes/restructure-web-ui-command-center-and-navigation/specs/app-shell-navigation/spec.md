## ADDED Requirements

### Requirement: Primary Navigation Uses Operational Domains
The Web UI shell SHALL organize top-level navigation around operator-facing domains: `Command Center`, `Jobs`, `Runs`, `Fleet`, `Integrations`, and `System`.

#### Scenario: Desktop shell shows the operational domains
- **WHEN** the desktop application shell renders
- **THEN** the primary navigation SHALL expose those operational domains as the main top-level choices
- **AND** low-frequency administrative pages SHALL NOT be the primary grouping for daily workflows

#### Scenario: Mobile shell preserves the same domain model
- **WHEN** the mobile application shell renders
- **THEN** the drawer or mobile navigation affordance SHALL expose the same top-level domains
- **AND** mobile users SHALL NOT lose access to any primary surface because of viewport size

### Requirement: Primary Object Routes Are Stable
Primary user-facing routes for core surfaces and objects SHALL use stable top-level paths instead of leading node-scoped path prefixes.

#### Scenario: Job detail has a stable route identity
- **WHEN** the user opens a job detail page
- **THEN** the route identity SHALL represent the job object itself via a stable top-level path
- **AND** any selected scope SHALL be represented separately from the object identity

#### Scenario: Run detail has a stable route identity
- **WHEN** the user opens a run detail page
- **THEN** the route identity SHALL represent the run object itself via a stable top-level path
- **AND** deep links SHALL remain meaningful without requiring the operator to parse a node-prefixed path

#### Scenario: Canonical route families remain predictable across major surfaces
- **WHEN** the operator navigates among Command Center, Jobs, Runs, Fleet, Integrations, and System
- **THEN** each surface SHALL use a canonical top-level route family
- **AND** scope-aware collection pages SHALL use explicit query or shell state instead of encoding scope in the leading path prefix

### Requirement: Scope Resolution Has Explicit Precedence Rules
The shell SHALL resolve effective scope using explicit precedence rules so collection pages and detail pages cannot silently disagree about what scope they are rendering.

#### Scenario: Collection route uses explicit scope before preferred scope
- **GIVEN** the operator has a persisted preferred scope
- **AND** they open a scope-aware collection route carrying an explicit `scope` query value
- **WHEN** the page resolves its effective scope
- **THEN** the page SHALL use the explicit route scope for that view
- **AND** the persisted preferred scope SHALL remain unchanged unless the user explicitly updates it

#### Scenario: Detail route resolves object scope before shell preference
- **GIVEN** the operator opens a stable object route such as a job, run, or fleet detail page
- **WHEN** the object is loaded
- **THEN** the object's own resolved scope SHALL govern object-specific data rendering
- **AND** a conflicting preferred scope SHALL NOT cause the page to misidentify the object as belonging to another scope

#### Scenario: Conflicting explicit scope is preserved only as return context
- **GIVEN** a detail page is opened with explicit context that does not match the object's resolved scope
- **WHEN** the page finishes loading
- **THEN** the page SHALL normalize object panels to the resolved object scope
- **AND** the conflicting explicit context SHALL be retained only for back-navigation or related-list context

### Requirement: Scope Selection Is Explicit And Persistent
The shell SHALL provide an explicit scope-selection affordance whose value persists across navigations and informs scope-aware pages.

#### Scenario: Scope changes persist across top-level pages
- **GIVEN** the operator selects a specific scope in the shell
- **WHEN** they navigate between top-level surfaces
- **THEN** the selected scope SHALL remain active until explicitly changed
- **AND** scope-aware pages SHALL render against that same effective scope unless the route overrides it

#### Scenario: Route-specific context can override preferred scope without mutating it silently
- **GIVEN** the operator has a persisted preferred scope
- **WHEN** they open a deep link that carries a different explicit scope context
- **THEN** the page SHALL honor the link's explicit scope for that view
- **AND** the preferred scope SHALL NOT be silently overwritten unless the user confirms or reselects it

### Requirement: The Shell Provides Contextual Secondary Navigation
The shell SHALL support contextual secondary navigation for the currently active top-level surface.

#### Scenario: Surface-specific secondary navigation renders for System
- **WHEN** the operator opens the `System` surface
- **THEN** the shell SHALL expose contextual secondary navigation for runtime, maintenance, appearance, about, or other System subsections
- **AND** the top-level navigation SHALL remain stable while the secondary navigation changes with context

#### Scenario: Mobile shell preserves contextual navigation without losing surface parity
- **WHEN** the operator opens the shell on mobile
- **THEN** the shell SHALL expose the same primary and contextual navigation model through drawer, tabs, or equivalent compact affordances
- **AND** mobile navigation SHALL NOT remove any top-level operational surface

### Requirement: Temporary Migration Aliases Preserve Context
The client router SHALL support a minimal set of temporary migration aliases for old internal primary-surface paths and normalize them to canonical stable routes without losing object identity or effective scope.

#### Scenario: Old internal job path normalizes to stable job route
- **GIVEN** the operator opens an old node-scoped job path that is still covered by migration aliases
- **WHEN** the route is resolved by the client router
- **THEN** the app SHALL normalize to the stable job route
- **AND** the resulting view SHALL preserve the referenced job and effective scope

#### Scenario: Old internal run path normalizes to stable run route
- **GIVEN** the operator opens an old node-scoped run path that is still covered by migration aliases
- **WHEN** the route is resolved by the client router
- **THEN** the app SHALL normalize to the stable run route
- **AND** the resulting view SHALL preserve the referenced run and effective scope

#### Scenario: Old node root normalizes to scoped collection view
- **GIVEN** the operator opens an old node root path that is still covered by migration aliases
- **WHEN** the route is resolved during migration
- **THEN** the app SHALL normalize to a canonical scoped collection route
- **AND** the normalization SHALL preserve whether the old node represented the hub or a specific agent
