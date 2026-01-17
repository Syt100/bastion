# Change: Tune filesystem picker path bar + footer UX (focus, icon weight, mobile badge)

## Why
After moving the filesystem picker’s actions into a more structured header/footer, a few UX issues remain:
- When opening the picker, the “Up” button can appear “selected” due to default focus placement, which is visually confusing.
- The spacing between the “Up” and “Refresh” icon buttons is slightly too large.
- The icon style feels a bit too bold/harsh.
- On mobile, placing the path actions on a separate row from the path input feels abrupt.
- On narrow mobile screens, showing `已选 x 项` in the footer competes with the confirm buttons.
- The “Current path” label is redundant and can be removed to reduce vertical noise.

## What Changes
- Focus behavior:
  - When the picker opens, focus the current-path input (not the “Up” button), avoiding a misleading “selected” appearance.
- Path bar layout:
  - Place “Up” and “Refresh” actions inline with the path input using the input prefix area to avoid a two-row layout on mobile.
  - Tighten spacing between the two icon buttons.
- Icon styling:
  - Keep using the existing icon library but render the icons with a lighter visual weight (thinner stroke / softer appearance).
- Footer selected-count:
  - On desktop, keep `已选 x 项` on the left side of the footer.
  - On mobile, avoid a separate selected-count text; show the count as a badge on the “Add selected” button to keep the footer on a single row.
- Copy:
  - Remove the “Current path” label (use placeholder / accessibility label instead).

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/i18n/locales/zh-CN.ts`
  - `ui/src/i18n/locales/en-US.ts`

## Compatibility / Non-Goals
- No backend API changes.
- No changes to picker semantics beyond the focus improvement and mobile selected-count presentation.

