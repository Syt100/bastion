# Change: Update Web UI Modern Visual Refresh and List UX (Colorful Theme, Toolbars, Dashboard Health)

## Why
The current Web UI is functional, but it reads as visually heavy and "industrial":
- Dense layouts, strong borders, and widespread glass surfaces reduce perceived quality and approachability.
- The palette is relatively muted and does not create a modern, fresh, colorful look.
- Key operational pages (Agents / Jobs / Snapshots) have crowded actions and inconsistent list patterns, which slows down daily use.

As the product grows, we need a cohesive design system and reusable list/toolbar components so UX remains consistent and iteration remains fast.

## What Changes
- Introduce a richer modern color system (light + dark) and a clearer surface hierarchy (page background -> surfaces -> elevated panels).
- Establish shared design tokens (colors/spacing/radii/shadows) and apply them to foundational components (AppShell, PageHeader, cards, tables, list rows, icon tiles).
- Create a standard ListToolbar + SelectionToolbar pattern and apply it to key list pages (Agents, Jobs, Snapshots, Bulk Operations, Notifications queue, Maintenance cleanup).
- Refactor page actions: one primary action per page, secondary actions grouped, destructive actions moved behind overflow menus with consistent confirmations.
- Upgrade Dashboard into a "status center" with top-level health summary and actionable links (failed runs, offline agents, notification failures, cleanup issues).
- Clarify node context semantics so users understand when the node picker affects the current view vs setting a preference.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/styles`, `ui/src/App.vue` theme overrides, layout/components, list views, and UI tests

