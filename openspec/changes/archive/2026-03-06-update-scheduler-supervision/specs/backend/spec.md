## ADDED Requirements

### Requirement: Critical Background Tasks Are Supervised
The backend SHALL supervise critical long-running background tasks so that unexpected task panics are detected rather than failing silently.

#### Scenario: Panic in a critical task is detected
- **GIVEN** a critical background task panics unexpectedly
- **WHEN** the panic occurs
- **THEN** the backend emits an error log identifying the task

### Requirement: Panic In A Critical Task Triggers Graceful Shutdown
When a supervised critical background task panics unexpectedly, the backend MUST trigger graceful shutdown via the shared cancellation token.

#### Scenario: Panic cancels shutdown token
- **GIVEN** the Hub is running normally
- **WHEN** a supervised critical background task panics unexpectedly
- **THEN** the shared shutdown token is cancelled so the server shuts down gracefully

