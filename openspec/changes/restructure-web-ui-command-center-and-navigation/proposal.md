## Why

The current Web UI is implementation-shaped rather than operator-shaped: the landing page shows flat overview cards, primary navigation is split between global and node-scoped URLs, and the shell does not clearly reflect the product's core operational workflows. That makes Bastion feel like a collection of pages instead of a backup control console.

## What Changes

- Replace the current Dashboard-first shell with an operational top-level navigation model centered on `Command Center`, `Jobs`, `Runs`, `Fleet`, `Integrations`, and `System`.
- Introduce a new Command Center landing page that prioritizes attention items, recent critical activity, and recovery readiness over generic KPI cards.
- Introduce stable top-level routes for primary objects and surfaces, while moving node scope to explicit scope selection and route/query context instead of leading path prefixes for the main experience.
- Add shared shell behaviors for desktop and mobile, including contextual secondary navigation, persistent scope state, and short-lived migration aliases for old internal paths.
- Expand the Web UI design-system contract to support a control-console visual hierarchy built around panels, rails, and attention-focused sections rather than repeated card grids.

## Capabilities

### New Capabilities
- `command-center`: operational landing page, aggregated attention model, recovery-readiness summary, and scope-aware command-center data contract
- `app-shell-navigation`: top-level information architecture, stable route model, scope-selection behavior, mobile/desktop shell parity, and short-lived route-normalization behavior during migration

### Modified Capabilities
- `web-ui-design-system`: extend the design-system requirements for control-console hierarchy, shell chrome, and mobile task-first density rules

## Impact

- Affected code:
  - `ui/src/router`, `ui/src/layouts`, `ui/src/components`, `ui/src/stores`, and related i18n/messages
  - `crates/bastion-http` and related aggregation/query code for a Command Center view model
  - supporting docs and navigation metadata
- Affected APIs:
  - new aggregated Command Center read model
  - stable route/deep-link handling for primary Web UI surfaces
- Cross-cutting considerations:
  - preserve mobile behavior while migrating shell/navigation
  - support phased migration from existing internal node-scoped entry points without committing to long-term legacy routes
