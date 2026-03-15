## ADDED Requirements

### Requirement: Dedicated Job Detail Routes Use Object-First Framing
Dedicated job detail routes SHALL visually privilege the selected job over collection-level controls.

#### Scenario: Detail route leads with the selected job
- **WHEN** an operator opens a dedicated job detail route directly or from the Jobs collection
- **THEN** the selected job header, object status, and object actions SHALL form the primary semantic anchor of the page
- **AND** collection-only controls SHALL NOT dominate the first visible viewport

#### Scenario: Collection context becomes compact return context
- **GIVEN** the operator arrived from a scoped or filtered Jobs collection
- **WHEN** the dedicated job detail route renders
- **THEN** the page SHALL preserve that context through a back action, scope chip, saved-view chip, or equivalent compact return affordance
- **AND** the page SHALL NOT require the full collection toolbar to remain visually primary just to preserve return context

### Requirement: Job Rows And Mobile Cards Expose A Clear Primary Open Action
Job rows and mobile cards SHALL make the primary "open this job" action obvious without sacrificing quick actions.

#### Scenario: Tapping or clicking the row opens the job clearly
- **WHEN** a job row or card is rendered
- **THEN** the operator SHALL have a clear primary target for opening the job detail
- **AND** quick actions such as `Run now` or `More` SHALL remain visually separate from the primary open target

#### Scenario: Touch actions are not icon-only for core job operations
- **WHEN** a mobile operator views a job card or dedicated job detail page
- **THEN** common actions such as opening, running, editing, or refreshing SHALL remain discoverable with text-backed affordances or equivalent explicit labeling
- **AND** icon-only controls SHALL NOT be the sole affordance for the primary task flow
