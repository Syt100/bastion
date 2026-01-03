## ADDED Requirements

### Requirement: Route-Driven Document Titles
The Web UI SHALL set `document.title` based on the active route using localized titles.

#### Scenario: Jobs page updates the browser title
- **WHEN** the user navigates to `/jobs`
- **THEN** the browser tab title reflects the localized Jobs title

### Requirement: Theme Color Tracks Light/Dark Mode
The Web UI SHALL update `meta[name="theme-color"]` to a sensible value for the current theme so mobile browser chrome matches the UI.

#### Scenario: Dark mode updates theme-color
- **WHEN** the user switches to dark mode
- **THEN** `meta[name="theme-color"]` is set to a dark surface color

### Requirement: Focus Visibility and Reduced Motion
The Web UI SHALL provide a visible `:focus-visible` outline style for keyboard navigation and SHOULD respect `prefers-reduced-motion`.

#### Scenario: Keyboard focus is visible
- **WHEN** the user navigates UI controls with the keyboard
- **THEN** focused controls show a clear focus-visible indicator

### Requirement: Shared Empty-State Pattern
The Web UI SHALL provide a shared empty-state component/pattern and SHOULD use it on key list screens.

#### Scenario: Agents page empty state is clear
- **WHEN** there are no agents to show
- **THEN** the page shows a consistent empty-state presentation indicating there is no data

### Requirement: Latest Request Wins for Rapid Refresh Screens
For screens that can trigger multiple requests quickly (filters/pagination), the Web UI SHALL ensure the latest request wins and SHOULD cancel previous requests via `AbortController`.

#### Scenario: Notification queue filters do not produce stale results
- **WHEN** the user changes notification queue filters rapidly
- **THEN** the UI displays results from the latest filter selection only

