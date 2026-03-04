## ADDED Requirements

### Requirement: Core Page Dialogs SHALL Reuse Shared Modal Shell
Core page dialogs SHALL reuse the shared modal shell so body spacing, footer actions, and scroll containment stay consistent across Jobs/Settings/Snapshots surfaces.

#### Scenario: Page-level dialogs render through shared shell
- **GIVEN** the user opens dialogs on Job Snapshots, Job Workspace, Bulk Operations, Settings Storage, Notifications Destinations, or Maintenance Cleanup pages
- **WHEN** dialog content and footer actions are rendered
- **THEN** the dialogs use the shared modal shell wrapper
- **AND** existing titles, header-extra actions, and submit/cancel behavior remain unchanged
