## ADDED Requirements

### Requirement: RunEventsBus Does Not Panic On Poisoned Mutex
The backend MUST tolerate `RunEventsBus` mutex poisoning and MUST continue operating without panicking when the bus lock is poisoned.

#### Scenario: Publish continues after a poisoned lock
- **GIVEN** a prior panic poisoned the RunEventsBus mutex
- **WHEN** the backend publishes a run event
- **THEN** publishing does not panic and the bus remains usable

#### Scenario: Subscribe continues after a poisoned lock
- **GIVEN** a prior panic poisoned the RunEventsBus mutex
- **WHEN** the backend subscribes to a run event stream
- **THEN** subscribing does not panic and a receiver is returned

