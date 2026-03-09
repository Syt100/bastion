---
## ADDED Requirements

### Requirement: Agent FS List Supports Pagination
The Hub↔Agent protocol SHALL support paginated filesystem listing so that the Hub can browse large Agent directories without transferring a full listing.

#### Scenario: Hub requests a page of directory entries from an Agent
- **GIVEN** an Agent is connected
- **WHEN** the Hub requests a filesystem list page with a `cursor` and `limit`
- **THEN** the Agent returns at most `limit` entries and a `next_cursor` when more entries exist

