# Change: Update Web UI Responsive Layout & Mobile UX

## Why
The Web UI has improved visually, but several UX issues remain:
- Responsive behavior is not working reliably (desktop/mobile nav overlap, layout not adapting).
- The brand icon looks visually compressed (too narrow).
- Header/content alignment diverges on wide screens.
- Many dialogs are too wide on desktop.
- UI copy punctuation is inconsistent (extra trailing periods in subtitles/help).

## What Changes
- Make the Web UI responsive using Tailwind standard breakpoints (mobile `< md`, desktop `>= md`).
- Use a single navigation pattern per breakpoint:
  - Mobile: top bar + hamburger + left drawer navigation.
  - Desktop: persistent left sidebar navigation (no drawer).
- Align header actions and main content to the same container width on wide screens.
- Render mobile-friendly card lists for Jobs/Agents/Settings instead of wide tables on small screens.
- Constrain modal widths on desktop to sensible maximums while keeping mobile usability.
- Replace the brand mark icon with Ionicons `ShieldCheckmark` (solid) and ensure it is not visually distorted.
- Standardize UI copy: subtitles and short helper texts omit trailing periods in both `zh-CN` and `en-US`.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` layout components, list views, and i18n strings

