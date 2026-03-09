## MODIFIED Requirements

### Requirement: Incomplete Cleanup Table Column Sizing Is Balanced
The web UI SHALL size columns on the incomplete cleanup list so that enum columns are compact and long-text columns have sufficient width for meaningful summaries.

#### Scenario: Enum columns are compact and error summary is readable
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the user views the incomplete cleanup list
- **THEN** the “目标” and “状态” columns are compact
- **AND** the “最近错误” column is wider than enum columns to show a readable summary (while remaining single-line truncated)

