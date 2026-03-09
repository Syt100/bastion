## ADDED Requirements

### Requirement: Follow Auto-Resumes When Returning To Bottom
The Web UI SHALL automatically re-enable follow mode when the user scrolls back to the bottom after follow was auto-disabled by scrolling away.

#### Scenario: Auto-disabled follow resumes when reaching bottom
- **GIVEN** follow mode was enabled
- **AND** follow was auto-disabled because the user scrolled away from the bottom
- **WHEN** the user scrolls back to the bottom of the Run Events list
- **THEN** follow mode is automatically re-enabled

#### Scenario: Manually disabled follow does not auto-resume
- **GIVEN** follow mode was manually disabled via the follow switch
- **WHEN** the user scrolls to the bottom of the Run Events list
- **THEN** follow mode remains disabled

