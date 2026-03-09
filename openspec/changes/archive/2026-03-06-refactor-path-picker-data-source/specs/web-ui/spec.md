---
## ADDED Requirements

### Requirement: Path Picker Is Data-Source Driven With Capability-Gated UI
The web UI SHALL provide a path picker implementation that is driven by a data source interface and a capability declaration so that new storage backends can reuse the picker UI without duplicating behavior.

#### Scenario: Filesystem browsing uses the generic picker
- **GIVEN** a filesystem data source backed by the existing filesystem list endpoint
- **WHEN** the user opens the filesystem picker
- **THEN** the picker uses the generic data-source driven path picker implementation

#### Scenario: Unsupported features are hidden or disabled
- **GIVEN** a data source that does not support a specific filter/sort/column
- **WHEN** the picker renders the filter and table UI
- **THEN** unsupported controls or columns are not shown (or are disabled) to prevent invalid requests

#### Scenario: A new storage backend can reuse the picker UI
- **GIVEN** a future WebDAV/S3 data source that implements the picker data source interface
- **WHEN** the UI adds a browser for that backend
- **THEN** the UI reuses the generic picker without rewriting the picker UI layout and interaction patterns
