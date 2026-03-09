## ADDED Requirements

### Requirement: HTTP Requests Have a Correlation ID
The backend SHALL assign a correlation/request ID to each inbound HTTP request and SHALL propagate it to responses to support debugging.

#### Scenario: Request-id is returned to the client
- **WHEN** a client sends an HTTP request without a request-id header
- **THEN** the backend generates a request ID
- **AND** includes it in the response headers

### Requirement: Logs and Spans Include Request Context
When emitting request-scoped logs/spans for HTTP requests, the backend SHALL include the request ID and relevant identifiers (e.g., `job_id`, `run_id`, `operation_id`) without leaking secrets.

#### Scenario: HTTP errors are diagnosable
- **WHEN** an HTTP request fails with a 4xx/5xx error
- **THEN** logs include the request-id and relevant identifiers to correlate client errors with backend logs
