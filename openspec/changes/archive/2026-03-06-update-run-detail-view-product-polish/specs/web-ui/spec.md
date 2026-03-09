## MODIFIED Requirements

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: Header uses localized status labels
- **GIVEN** the UI is displayed in Chinese
- **WHEN** the run status is `success`
- **THEN** the Run Detail header shows “成功” (not the raw enum value)

#### Scenario: Target type labels are productized
- **GIVEN** a run summary includes a target type such as `local_dir`
- **THEN** the Run Detail page displays a user-friendly label (e.g. “本地目录”) instead of the raw identifier

#### Scenario: Progress help affordance is compact
- **GIVEN** the run progress panel is visible
- **THEN** stage help uses a compact help icon (e.g. a help-circle)
- **AND** the panel avoids visually repetitive stage rows on the primary view
