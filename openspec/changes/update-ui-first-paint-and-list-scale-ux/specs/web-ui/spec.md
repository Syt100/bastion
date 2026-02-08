## ADDED Requirements

### Requirement: Dashboard Provides First-Paint Skeleton and Viewport-Lazy Chart Rendering
The dashboard SHALL provide immediate first-paint skeleton loading placeholders for key summary sections and SHALL defer rendering the trend chart component until the chart container is near or inside the viewport.

#### Scenario: First dashboard paint shows skeleton placeholders
- **GIVEN** a user opens the dashboard and overview data is still loading
- **WHEN** the dashboard first renders
- **THEN** summary sections render skeleton placeholders instead of empty or misleading zero values
- **AND** loading UI remains responsive until overview data is available

#### Scenario: Trend chart render is deferred by viewport visibility
- **GIVEN** dashboard trend data exists
- **WHEN** the chart section has not yet entered the viewport
- **THEN** heavy chart rendering is deferred
- **AND** once the chart section enters (or nears) the viewport, the chart renders with loading fallback

### Requirement: Stores Use Shared Latest-Request Cancellation Semantics
Web UI stores that refresh list/overview data SHALL use a shared latest-request cancellation mechanism so that superseded in-flight requests are canceled and stale responses cannot overwrite newer state.

#### Scenario: Overlapping dashboard refresh only keeps latest response
- **GIVEN** dashboard refresh request A is in flight
- **AND** request B starts before A completes
- **WHEN** request B completes successfully
- **THEN** request A is canceled or treated stale
- **AND** dashboard state reflects request B only

### Requirement: Jobs and Agents Views Support Paged Rendering
Jobs and agents list presentations SHALL support client-side pagination to limit rows/cards rendered at once while preserving filter/sort semantics.

#### Scenario: Large filtered result set renders current page only
- **GIVEN** a filtered list returns many rows
- **WHEN** the user views jobs or agents list
- **THEN** only items for the selected page are rendered
- **AND** pagination controls allow changing page and page size
