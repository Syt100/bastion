## ADDED Requirements

### Requirement: UI Unit Tests Avoid Stub-Generated Native Prop Warnings
When UI unit tests stub third-party components with native elements, test stubs SHALL avoid forwarding invalid native-only props that produce avoidable Vue runtime warnings.

#### Scenario: Stubbed input does not emit invalid native size warning
- **GIVEN** a UI unit test stubs an input-like component with a native `<input>` element
- **WHEN** the test renders a view that passes third-party component props (for example, `size=\"small\"`)
- **THEN** the stub handles or filters that prop safely
- **AND** the test run does not emit an avoidable warning for invalid native input size assignment
