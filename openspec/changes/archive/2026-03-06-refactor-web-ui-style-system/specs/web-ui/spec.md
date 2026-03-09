## ADDED Requirements

### Requirement: Shared Style Utilities for Common UI Patterns
The Web UI SHALL provide shared style utilities for frequently reused UI patterns to reduce duplication and keep visuals consistent.

Shared style utilities SHOULD cover:
- navigation chrome “glass” surfaces,
- settings-like list row hover/spacing,
- muted helper text,
- icon tiles used in list items.

#### Scenario: List row style is consistent across settings-like lists
- **WHEN** the UI renders settings-like lists (e.g. Settings overview, Notifications overview)
- **THEN** list rows share consistent spacing, hover behavior, and icon tile presentation via shared style utilities

### Requirement: Unused Legacy Views Are Removed
The Web UI SHALL remove unused legacy view files that are not referenced by the router/tests to reduce confusion and accidental regressions.

#### Scenario: No unused Settings legacy view remains
- **WHEN** the router is inspected
- **THEN** there is no unused legacy Settings view implementation file in the codebase

