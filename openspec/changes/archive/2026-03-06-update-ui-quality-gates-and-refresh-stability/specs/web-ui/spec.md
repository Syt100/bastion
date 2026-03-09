## ADDED Requirements

### Requirement: List Refresh Uses Latest-Request-Wins Semantics
For list refresh workflows in Web UI state stores, the system SHALL apply latest-request-wins semantics:
- when multiple refresh requests overlap, only the most recent request may update list data and loading state
- stale request failures SHALL be ignored and MUST NOT overwrite newer successful state

#### Scenario: Stale success response cannot overwrite newer result
- **GIVEN** a list view triggers refresh request A
- **AND** before A returns, the user triggers refresh request B
- **AND** request B completes successfully first with dataset B
- **WHEN** request A completes later with dataset A
- **THEN** the store keeps dataset B
- **AND** the stale dataset A is ignored

#### Scenario: Stale failure cannot override newer success
- **GIVEN** a list view triggers refresh request A
- **AND** before A returns, the user triggers refresh request B
- **AND** request B completes successfully first
- **WHEN** request A fails later
- **THEN** the store keeps the successful state from request B
- **AND** the stale failure from request A does not become the active list state
