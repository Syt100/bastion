## Why

Live UI review of the rebuilt `Command Center` and `Jobs` surfaces shows that the broad structural goals landed, but several operator-critical quality gaps remain:

- hero and summary copy is still too slogan-like for a control console
- command-center hierarchy is improved, but neutral metadata still competes with actionable states
- job detail still feels partially embedded in collection chrome instead of reading like a first-class object view
- mobile job authoring preserves too much desktop stepper structure, especially in step navigation and side summaries
- touch workflows still rely on weak primary affordances in a few list and detail states

These are not new product directions. They are follow-up quality requirements needed to make the existing information architecture feel deliberate, professional, and operationally trustworthy.

## What Changes

- Add explicit copy/tone requirements for operator-facing control-console surfaces.
- Tighten `Command Center` hierarchy so risk, readiness, and next actions dominate over neutral counters and timestamps.
- Tighten `Jobs` object framing so dedicated job detail flows emphasize the selected job rather than collection-level controls.
- Define mobile-specific step navigation and summary-collapse behavior for the job editor instead of inheriting desktop structure.
- Add touch-target and action-prominence requirements for list rows, detail actions, and mobile operations.

## Capabilities

### Modified Capabilities
- `command-center`: add hierarchy and copy-quality constraints for the landing surface
- `jobs-workspace`: add object-first detail framing and stronger touch/action affordances
- `job-editor-flow`: add compact mobile step navigation and collapsible summary behavior
- `web-ui-design-system`: add professional operational copy and stronger action/readability rules

## Impact

- Affected code:
  - `ui/src/views/dashboard`, `ui/src/views/jobs`, shell and shared layout primitives, and related i18n/messages
  - any shared status, badge, or panel primitives used by those pages
- Product impact:
  - current top-level operational pages read more like a professional control plane and less like a generic dashboard
  - desktop and mobile operator workflows become easier to scan and safer to act on
