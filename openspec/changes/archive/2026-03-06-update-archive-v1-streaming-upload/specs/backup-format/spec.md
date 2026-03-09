## ADDED Requirements

### Requirement: Archive Parts Can Be Stored Incrementally
For `archive_v1`, the system SHALL support storing each finalized `payload.part*` to the configured target as soon as it is finalized, without waiting for packaging to finish.

After a part is stored successfully, the system SHALL remove the local staging file for that part.

#### Scenario: A finalized part is stored and deleted before packaging completes
- **GIVEN** an `archive_v1` filesystem backup run produces multiple `payload.part*` files
- **WHEN** the run finalizes `payload.part000001`
- **THEN** the system stores `payload.part000001` to the target
- **AND** the local staging file for `payload.part000001` is removed even if later parts are still being packaged

### Requirement: Rolling Part Storage Applies Backpressure
When storing finalized parts incrementally, the system SHALL apply backpressure so completed parts do not accumulate unboundedly on local disk.

#### Scenario: Target is slower than packaging
- **GIVEN** an `archive_v1` filesystem backup run is packaging data faster than the target can accept parts
- **WHEN** the internal queue of finalized parts is full
- **THEN** packaging waits before finalizing additional parts
