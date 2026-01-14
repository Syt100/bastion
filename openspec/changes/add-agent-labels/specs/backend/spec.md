## ADDED Requirements

### Requirement: Agents Support Persistent Labels
The backend SHALL allow each agent to have zero or more labels and SHALL persist labels in the database.

#### Scenario: Labels round-trip
- **GIVEN** an agent exists
- **WHEN** labels are added to the agent
- **THEN** listing or fetching the agent MUST return those labels
- **AND** labels MUST remain after a Hub restart

### Requirement: Label Constraints and Validation
The backend SHALL validate labels to reduce ambiguity and operator error.

#### Scenario: Invalid label is rejected
- **WHEN** a user attempts to add a label with invalid format
- **THEN** the API MUST return an error indicating the label is invalid

### Requirement: Agents List Supports Label Filtering With AND/OR Semantics
The backend SHALL support filtering agents by labels using:
- `labels[]`: repeated query parameter values
- `labels_mode`: `and|or` (default `and`)

#### Scenario: AND mode returns agents that contain all labels
- **GIVEN** agent A has labels `prod` and `shanghai`
- **AND** agent B has label `prod` only
- **WHEN** the user lists agents with `labels[]=prod&labels[]=shanghai&labels_mode=and`
- **THEN** the response MUST include agent A
- **AND** MUST NOT include agent B

#### Scenario: OR mode returns agents that contain any label
- **GIVEN** agent A has labels `prod` and `shanghai`
- **AND** agent B has label `prod` only
- **WHEN** the user lists agents with `labels[]=prod&labels[]=shanghai&labels_mode=or`
- **THEN** the response MUST include agent A
- **AND** MUST include agent B

### Requirement: API Provides Label Index For UI
The backend SHALL provide an authenticated API that lists known labels with usage counts to support UI autocomplete and filtering.

#### Scenario: Label index is returned with counts
- **GIVEN** multiple agents have labels
- **WHEN** the user requests the label index
- **THEN** the response MUST include the distinct labels and their usage counts

### Requirement: Authentication and CSRF Protection
All label modification APIs SHALL require an authenticated session and CSRF protection.

#### Scenario: Unauthenticated user cannot modify labels
- **WHEN** an unauthenticated user attempts to add/remove/set labels
- **THEN** the request MUST be rejected

