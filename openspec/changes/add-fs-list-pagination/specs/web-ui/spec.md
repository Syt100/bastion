---
## MODIFIED Requirements

### Requirement: Filesystem Picker Supports Huge Directories
The web UI filesystem picker SHALL support browsing directories with a very large number of entries by using paged listing and server-side filtering.

#### Scenario: Browse a huge directory without loading all entries at once
- **GIVEN** the filesystem picker is opened on a directory with many entries
- **WHEN** the picker loads the listing
- **THEN** the picker fetches a bounded page of entries from the server
- **AND** the UI remains responsive
- **AND** the user can load additional pages on demand

#### Scenario: Filters re-fetch results (server-side)
- **GIVEN** the filesystem picker shows a paged listing
- **WHEN** the user changes search/type/size filters
- **THEN** the picker re-fetches the listing from the server with the new filter parameters
- **AND** the result represents the whole directory (not only the already-loaded rows)

