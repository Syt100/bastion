---
## MODIFIED Requirements

### Requirement: Picker Modals Use A Single Scroll Region
The web UI SHALL render picker modals (filesystem picker and restore-entry picker) such that:
- the modal content does not introduce an additional vertical scrollbar on top of the table,
- the file/entry table fills the remaining available height between the picker header controls and the modal footer, and
- the table is the primary scroll container for long lists.

#### Scenario: Desktop picker does not show a double scrollbar
- **GIVEN** the user opens a picker modal on a desktop-sized screen
- **WHEN** the file/entry listing contains many rows
- **THEN** the user scrolls the listing using a single scrollbar (the table)
- **AND** the modal body does not introduce an additional vertical scrollbar

#### Scenario: Mobile picker does not leave a large blank gap below the table
- **GIVEN** the user opens a picker modal on a mobile-sized screen
- **WHEN** the picker renders its listing
- **THEN** the table uses the remaining available height in the modal body
- **AND** the picker does not leave a large blank gap between the table and the modal footer

