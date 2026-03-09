## ADDED Requirements

### Requirement: Dashboard Defers Heavy Desktop Recent-Runs Table Module
The dashboard SHALL defer loading the desktop recent-runs table module until the recent-runs section is near or inside the viewport.

#### Scenario: Desktop recent table module loads on visibility
- **GIVEN** the dashboard has recent run records
- **AND** viewport is desktop-sized
- **WHEN** the recent section has not yet entered viewport proximity
- **THEN** the heavy desktop table module is not rendered yet
- **AND** the dashboard shows a lightweight deferred-loading placeholder for that section
- **WHEN** the section enters viewport proximity
- **THEN** the desktop table module is loaded and rendered

### Requirement: Dashboard Uses Shared Viewport-Lazy Activation Helper
Dashboard viewport-lazy section activation logic SHALL use a shared helper to keep behavior consistent and reduce duplicated observer lifecycle code.

#### Scenario: Shared helper activates section once visible and handles cleanup
- **GIVEN** a section is configured with viewport-lazy activation
- **WHEN** the target element intersects the viewport
- **THEN** the section is marked ready and observer is disconnected
- **AND** helper cleanup disconnects observers when no longer needed

### Requirement: Deferred Dashboard Sections Show Lightweight Animated Loading Feedback
When deferred dashboard sections are waiting for module activation, the UI SHALL display lightweight animated loading feedback in addition to skeleton placeholders.

#### Scenario: Deferred section placeholder includes animated loading indicator
- **GIVEN** a deferred dashboard section is not yet ready to render
- **WHEN** the placeholder is shown
- **THEN** users can see an animated loading indicator
- **AND** no heavy animation assets are required to render the indicator
