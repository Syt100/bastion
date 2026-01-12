## MODIFIED Requirements

### Requirement: Desktop Data Tables Support Persistent Column Resizing
The web UI SHALL allow operators to manually resize columns on desktop list tables. Resized column widths SHALL persist across page refresh and SHALL be isolated per list page.

#### Scenario: Resized width persists across refresh
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user resizes a column on a list page
- **AND** refreshes the browser page
- **THEN** the list table renders with the previously resized column width

#### Scenario: Column widths are isolated per list page
- **GIVEN** the user resized columns on the incomplete cleanup list page
- **WHEN** the user navigates to the notifications queue list page
- **THEN** the notifications queue table uses its own stored widths (or defaults)
- **AND** does not reuse widths from the incomplete cleanup page

