## ADDED Requirements

### Requirement: Jobs Overview Shows Configuration Metadata Cards
In the Jobs workspace Overview section, the Web UI SHALL present the job's key configuration metadata as compact summary cards.

The metadata SHALL include at least:
- source type,
- target type,
- backup format, and
- encryption.

#### Scenario: Overview displays configuration metadata with visual encoding
- **GIVEN** the user is viewing a job Overview
- **WHEN** the job has a defined spec
- **THEN** the UI shows cards for source type, target type, backup format, and encryption
- **AND** each card uses tags and/or text color to make the values scannable

#### Scenario: Cards remain usable on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the Overview is rendered
- **THEN** the metadata cards stack without horizontal overflow

### Requirement: Overview Does Not Provide Quick Link Shortcuts
The Jobs workspace Overview section SHALL NOT provide a dedicated “Quick links” block for navigating to History/Data.

#### Scenario: Navigation relies on section tabs
- **GIVEN** the user is viewing a job workspace
- **WHEN** the user wants to access History or Data
- **THEN** the user uses the job section navigation (Overview/History/Data)
