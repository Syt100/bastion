# Change: Refactor UI visual foundation and density

## Why
Recent UX refactors improved interaction consistency, but page aesthetics still show uneven hierarchy, excessive visual noise, and inconsistent spacing/density between list pages and detail surfaces. This reduces scanability and makes the UI feel less cohesive.

## What Changes
- Introduce explicit visual hierarchy tokens/classes for page title, section title, and metadata text.
- Reduce visual noise by tuning page background intensity, card chrome, borders, and surface layering.
- Standardize vertical spacing rhythm for list scaffolds, toolbars, and pagination blocks.
- Add shared density/metadata styles for list rows and table secondary information.
- Migrate core pages (`JobsWorkspaceShellView`, `AgentsView`, and shared list components) to the new visual baseline.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/styles/main.css`
  - `ui/src/components/PageHeader.vue`
  - `ui/src/components/list/ListPageScaffold.vue`
  - `ui/src/components/list/ListToolbar.vue`
  - `ui/src/components/list/AppPagination.vue`
  - `ui/src/views/jobs/JobsListRowItem.vue`
  - `ui/src/views/jobs/useJobsTableColumns.ts`
  - `ui/src/views/AgentsView.vue`

## Non-Goals
- Introducing new page-level features or backend behavior changes.
- Replacing existing theme presets with a brand-new theme system.
