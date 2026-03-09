## ADDED Requirements

### Requirement: WebDAV Upload Traffic Is Bounded By Configured Limits
When uploading backup payloads to WebDAV, the system SHALL enforce configured request limits to avoid overwhelming the target server.

Limits SHALL include:
- a maximum concurrency (in-flight requests)
- a maximum request rate (requests per second), at least for `PUT`

#### Scenario: Concurrency is capped
- **WHEN** a raw-tree backup uploads many files to WebDAV
- **THEN** the number of concurrent WebDAV requests does not exceed the configured limit

#### Scenario: Completion remains atomic under limiting
- **WHEN** WebDAV request limits are enabled
- **THEN** the `complete` marker is still written last

