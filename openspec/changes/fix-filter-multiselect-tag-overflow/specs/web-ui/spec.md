---
## MODIFIED Requirements

### Requirement: Enum Filters Support Multi-Select
The web UI SHALL allow selecting multiple values for low-cardinality enum filters.

#### Scenario: Multi-select filters remain compact with many selections
- **GIVEN** the user selects many values in a multi-select filter
- **THEN** the filter control remains compact (does not grow vertically to show all tags)
- **AND** the user can still inspect all selected values via an overflow indicator/popover

