## ADDED Requirements

### Requirement: Modern Colorful Theme Tokens
The Web UI SHALL define a richer set of shared design tokens (light + dark) for background/surfaces/text/accent/status colors so the UI looks modern, fresh, and consistent.

#### Scenario: A single token update changes the accent color across the UI
- **WHEN** the primary accent color token is updated
- **THEN** buttons, links, active navigation states, and primary highlights consistently reflect the updated accent color

### Requirement: Clear Surface Hierarchy With Reduced Visual Weight
The Web UI SHALL present a clear surface hierarchy (page background -> surfaces -> elevated panels) and SHALL reduce the heavy/industrial look by avoiding excessive borders and blur effects.

#### Scenario: Content surfaces are readable without heavy borders
- **WHEN** a standard content page (e.g. Agents or Jobs) is rendered
- **THEN** primary content is placed on solid surfaces (cards/panels)
- **AND** glass/blur is limited to navigation chrome, not the main content area

### Requirement: Refreshed Navigation Chrome With Strong Active State
The Web UI navigation chrome (desktop sider + header) SHALL adopt the refreshed theme and SHALL make the active section obvious using modern color cues (tint/indicator) rather than heavy borders.

#### Scenario: Active navigation item is clearly visible
- **WHEN** the user navigates between Dashboard, Jobs, and Agents
- **THEN** the active menu item is visually distinct via the shared theme (e.g. tint/indicator)

### Requirement: Consistent Page Header Action Layout
The Web UI SHALL standardize page header layout so that actions are easy to scan and do not overflow on mobile.

#### Scenario: Header actions collapse on mobile instead of wrapping
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the page header has multiple actions
- **THEN** non-critical actions are presented via an overflow menu instead of wrapping into multiple rows

### Requirement: Action Hierarchy and Safer Destructive Actions
The Web UI SHALL enforce a consistent action hierarchy:
- each page has at most one primary action,
- secondary actions are grouped,
- destructive actions are placed behind an overflow menu and require consistent confirmation.

#### Scenario: Destructive actions are not the most prominent controls
- **WHEN** a list row includes a destructive operation (e.g. revoke agent, delete)
- **THEN** the destructive operation is not a primary button in the row
- **AND** it requires an explicit confirmation step

### Requirement: Standard ListToolbar Pattern
The Web UI SHALL provide a shared ListToolbar pattern for list screens to support:
- search,
- filters,
- sort,
- view toggle (table/cards when applicable),
- refresh,
- and a primary action.

#### Scenario: List pages share the same toolbar layout
- **WHEN** the user opens Agents, Jobs, or Snapshots
- **THEN** each page presents the same core toolbar pattern (with page-specific controls) in a consistent location and layout

### Requirement: Standard SelectionToolbar for Bulk Actions
When list selection is available, the Web UI SHALL show a SelectionToolbar when one or more items are selected.

#### Scenario: Selecting rows reveals bulk actions
- **WHEN** the user selects one or more rows in a list
- **THEN** a selection toolbar appears showing the selected count
- **AND** bulk actions are discoverable without scrolling back to the page header

### Requirement: Agents List Efficiency Improvements
The Agents page SHALL improve efficiency by providing:
- search (name/id),
- quick status filters (online/offline/revoked),
- and a simplified per-row action set with overflow for secondary/danger actions.

#### Scenario: Operator finds an offline agent quickly
- **WHEN** the operator filters for offline agents and searches by name/id
- **THEN** the list updates immediately to show matching agents
- **AND** the operator can open details or agent-scoped pages with minimal clicks

### Requirement: Jobs List Efficiency Improvements
The Jobs list SHALL improve efficiency by providing:
- search (job name),
- filters (archived/type as applicable),
- sort (e.g. updated time/name),
- and consistent primary/secondary actions.

#### Scenario: Archived jobs are easy to find without visual clutter
- **WHEN** the user toggles "show archived" and searches by job name
- **THEN** archived jobs are clearly indicated
- **AND** primary actions that should not apply (e.g. Run now) are not presented as enabled

### Requirement: Snapshots List Efficiency Improvements
The Snapshots page SHALL improve efficiency by:
- supporting cursor-based pagination ("Load more"),
- providing filters (status/pinned/target),
- and making pinned snapshots visually distinct and protected from accidental deletion.

#### Scenario: Pinned snapshots are protected during bulk delete
- **WHEN** the user opens the bulk delete confirmation
- **THEN** pinned snapshots are clearly highlighted
- **AND** deletion requires an explicit extra acknowledgement when pinned items are included

### Requirement: Dashboard Health Summary and Actionable Links
The Dashboard SHALL act as a status center by showing a top-level health summary with actionable links to remediation screens.

#### Scenario: Operator can navigate from a health card to the relevant screen
- **WHEN** the Dashboard shows an offline agents or notification failures indicator
- **THEN** the indicator provides a direct navigation path to the relevant page with appropriate filters applied

### Requirement: Node Context Semantics Are Clear
The Web UI SHALL clearly communicate node context so users understand whether they are viewing global pages or node-scoped pages.

#### Scenario: Node picker behavior is clear on global pages
- **WHEN** the user changes the node selection while on a global page (e.g. Dashboard)
- **THEN** the UI communicates that it updates the preferred node (used for future node-scoped navigation)
- **AND** it does not silently change the current page scope unless the page is node-scoped

### Requirement: Visual Accessibility Remains Intact After Refresh
The refreshed visual design SHALL preserve focus-visible indicators, SHOULD maintain sufficient color contrast in light/dark mode, and SHOULD respect reduced motion preferences.

#### Scenario: Keyboard focus remains clearly visible after theme changes
- **WHEN** the user navigates the UI with the keyboard
- **THEN** the focused control shows a clearly visible focus indicator under the refreshed theme

