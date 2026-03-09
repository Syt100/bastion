## ADDED Requirements

### Requirement: Run event hint labels are localized
The Web UI SHALL localize run-event detail hint labels for all supported locales.

#### Scenario: Chinese locale renders localized hint label
- **GIVEN** UI locale is `zh-CN`
- **AND** run event detail contains a `hint` field
- **WHEN** user opens event details
- **THEN** the hint label SHALL be displayed in Chinese instead of hardcoded English text

### Requirement: Hint rendering is source-agnostic
The Web UI SHALL render hint text whenever `hint` is present, regardless of whether it originates from run failures or cleanup maintenance events.

#### Scenario: Cleanup event provides hint field
- **GIVEN** a run event from cleanup/maintenance pipeline includes `fields.hint`
- **WHEN** user opens run event details
- **THEN** UI SHALL render the hint block with localized label and original hint content
