---
## ADDED Requirements

### Requirement: Agent FS List Supports Sorting
The Hub↔Agent filesystem list protocol SHALL support sorting directory entries to enable consistent UX when browsing large directories.

#### Scenario: Hub requests a sorted page
- **GIVEN** an Agent is connected
- **WHEN** the Hub requests a filesystem list page with `sort_by` and `sort_dir`
- **THEN** the Agent returns entries ordered by the requested sort
- **AND** the paging cursor remains stable for that sort order
