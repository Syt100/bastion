---
## ADDED Requirements

### Requirement: Picker Modals Use Shared Layout Building Blocks
The web UI SHALL implement picker modals using shared layout building blocks to reduce duplication and keep UX consistent across pickers.

#### Scenario: A shared layout prevents “fix one modal, forget the other”
- **GIVEN** the filesystem picker and restore entries picker share common UX elements (search, filters, table, footer)
- **WHEN** a UX fix is applied to the shared picker layout
- **THEN** both pickers reflect the fix without requiring duplicated changes
