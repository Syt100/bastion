# Design: Web UI Responsive Layout & Mobile Card Lists

## Breakpoints
Use Tailwind defaults:
- Mobile: `< md` (viewport width < 768px)
- Desktop: `>= md` (viewport width ≥ 768px)

## Navigation & Layout

### Desktop (`>= md`)
- Persistent left sidebar navigation is visible.
- Header contains only global actions (language/theme/logout) aligned to the same container width as page content.
- The brand mark (logo) is placed in the sidebar header area.
- No hamburger button and no drawer navigation.

### Mobile (`< md`)
- No persistent sidebar.
- Header contains: hamburger + brand mark + global actions (may collapse to a menu later if needed).
- Hamburger opens a left drawer navigation with the same menu items as desktop.

### Implementation Note
Avoid relying on Tailwind `hidden/md:block` classes directly on Naive UI component roots (their internal styling may override `display`).
Prefer:
- Wrapping Naive UI components in plain `div` containers that carry responsive classes, or
- `v-if` layout branching using a reactive `isMobile/isDesktop` computed derived from `matchMedia`.

## Mobile Card Lists (instead of tables)
For Jobs/Agents/Settings list pages:
- Desktop: keep `NDataTable`.
- Mobile: render a list of cards, one item per card:
  - Title row: primary identifier (e.g., job name) + status tag(s) if applicable.
  - Body: 2–4 key fields as labeled rows.
  - Footer: primary action (e.g., Run now / Edit) and a secondary “More” menu for additional actions.

## Modal Sizing
Define a shared set of modal width presets for desktop, each still responsive to mobile width:
- Small: `min(560px, calc(100vw - 32px))`
- Medium: `min(720px, calc(100vw - 32px))`
- Large: `min(980px, calc(100vw - 32px))`
Keep the Jobs create/edit modal as the large preset (or larger if needed) due to its multi-step content.

## Brand Icon
Use Ionicons `ShieldCheckmark` (solid) for the brand mark icon.
Ensure the icon is visually balanced at small sizes:
- Prevent flex shrink on the icon container.
- Prefer a square `aspect-ratio` box for consistent rendering.

## UI Copy Punctuation
For subtitles and short helper texts:
- Remove trailing periods (`。` / `.`) in both `zh-CN` and `en-US`.
For warnings/alerts with complete sentences:
- Keep punctuation as needed for readability.

