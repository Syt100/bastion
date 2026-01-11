## MODIFIED Requirements

### Requirement: Incomplete Cleanup “最近错误” Is Compact And Accessible
The web UI SHALL render the incomplete cleanup list “最近错误” column as a single-line truncated summary that does not increase row height. The full error SHALL remain accessible from the list.

#### Scenario: Long errors do not widen the table
- **GIVEN** an incomplete cleanup task has a long last error string
- **WHEN** the user views the incomplete cleanup list on a desktop screen
- **THEN** the “最近错误” cell shows a single-line truncated summary
- **AND** the table layout does not expand to fit the full error text

#### Scenario: User can view the full error from the list
- **GIVEN** an incomplete cleanup task has a recorded last error
- **WHEN** the user hovers the “最近错误” cell
- **THEN** the UI shows the full error text
- **WHEN** the user clicks the “最近错误” cell
- **THEN** the UI provides access to the full error text (e.g. opens the details view)

