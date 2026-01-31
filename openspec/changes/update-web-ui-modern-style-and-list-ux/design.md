# Design: Modern Web UI Visual Refresh + List UX

This change focuses on three layers, implemented in this order:
1) Visual foundations (design tokens + refreshed chrome/surfaces)
2) Shared list UX components (ListToolbar / SelectionToolbar / overflow actions)
3) Page-level IA/UX updates (Agents/Jobs/Snapshots + Dashboard health + node context clarity)

## Design Goals
- Modern + fresh: richer accent colors, clearer hierarchy, less "console" feeling.
- Consistent: shared patterns for surfaces, headers, toolbars, tables/cards, and actions.
- Efficient: fewer clicks for common tasks; predictable list behaviors; safe destructive actions.
- Accessible: maintain focus-visible, sufficient contrast, and respect reduced motion.

## Visual System (Tokens)
### Token categories
Define tokens as CSS variables (light + dark) and map key Naive UI theme overrides to them.
- Background/surfaces: `--app-bg`, `--app-surface`, `--app-surface-2`, `--app-border`
- Text: `--app-text`, `--app-text-muted`
- Brand accents (richer than today, but still B2B): `--app-primary`, `--app-primary-2`, `--app-accent-*`
- Status: `--app-success`, `--app-warning`, `--app-danger`, `--app-info`
- Shadows/blur: `--app-shadow-*`, `--app-glass-blur` (use sparingly)

### Proposed palette (initial; subject to tweak during implementation)
The goal is "fresh + modern" with richer accents while staying B2B-appropriate.

Light mode:
- Primary: blue `#3b82f6`
- Primary-2: cyan `#06b6d4` (used for gradients/secondary accents)
- Accent: indigo `#6366f1` / purple `#8b5cf6` (used sparingly for highlights)
- Success: `#22c55e`, Warning: `#f59e0b`, Danger: `#ef4444`, Info: `#0ea5e9`
- Background: very light neutral with subtle color wash (e.g. `#f8fafc` + soft gradients)

Dark mode:
- Background: deep blue `#0b1220` (already used) with a subtle color wash
- Surfaces: slightly elevated blue-grays (avoid pure black)
- Keep accents slightly brighter to preserve contrast against dark surfaces

### Visual hierarchy rules
- The page background SHOULD carry subtle color (e.g., soft gradient) to avoid "flat gray".
- Most content SHOULD sit on solid surfaces (cards/panels) rather than blurred glass.
- Glass/blur SHOULD be limited to navigation chrome (top bar / sider) to reduce visual noise and GPU cost.

## Navigation Chrome
- Desktop sider: visually lighter, clearer active state (colored indicator or background tint), less border weight.
- Top header: subtle background with a small accent (e.g., gradient hairline) and compact global actions.
- Selected menu item SHOULD be obvious without relying on heavy borders.

## Page Header / Actions
Standardize action hierarchy and responsiveness:
- Each page has ONE primary action (if any), visually prominent.
- Secondary actions (Refresh, Browse, etc.) are grouped.
- Destructive actions live in an overflow menu, with consistent confirmation copy.
- On mobile, non-critical header actions collapse into overflow to avoid wrapping.

## List UX Components
### ListToolbar (shared)
Responsibilities:
- Provide a consistent layout for: search, filters, sort, view toggle (table/cards), refresh, and primary action.
- Desktop: horizontal, compact.
- Mobile: stacked layout; actions collapse into overflow.

### SelectionToolbar (shared)
- Appears when selection count > 0.
- Shows selected count + quick actions (Clear selection, bulk actions).
- Clarifies scope: "selected rows" vs "current filters".

### Overflow actions
Use a consistent dropdown menu pattern:
- Secondary actions in overflow.
- Destructive actions grouped under a divider and styled as danger.

## Page-Level Notes
- Agents: reduce row-level button clutter; prefer "Open" + overflow actions; add search and quick status filters.
- Jobs: add search/filter/sort; row click opens detail; actions follow the hierarchy above.
- Snapshots: switch to cursor-based pagination ("Load more"); add filters (status/pinned/target); pinned items are easy to identify and protected.
- Dashboard: add top "Health" summary row with links to remediation surfaces.
- Node context: visually clarify when a page is node-scoped; node picker labeling reflects whether it changes current view or only preference.
