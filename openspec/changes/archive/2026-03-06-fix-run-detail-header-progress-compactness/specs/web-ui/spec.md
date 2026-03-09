## MODIFIED Requirements

### Requirement: Run Detail Header and Overview Are Productized
The Run Detail page SHALL present productized labels and layouts.

#### Scenario: Status badge is placed on the right
- **GIVEN** the Run Detail header is visible
- **THEN** the status badge is rendered on the right side of the header (separate from the title)

#### Scenario: Long target paths are readable
- **GIVEN** the run has a long target path
- **THEN** the overview section shows the full path (wrapping to multiple lines)
- **AND** it does not truncate the path with an ellipsis

### Requirement: Progress Panel Help and Stage Completion
The Progress panel SHALL keep help and stage completion states readable.

#### Scenario: Scan/Packaging help remains accessible
- **GIVEN** the Progress panel shows stages
- **THEN** help for Scan and Packaging is accessible via a compact help affordance

#### Scenario: Upload at 100% is rendered as finished
- **GIVEN** the current stage is Upload
- **WHEN** Upload progress reaches 100%
- **THEN** the stage is rendered as finished (no in-progress stage progress bar)
