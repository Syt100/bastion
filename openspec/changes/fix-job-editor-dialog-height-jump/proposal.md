# Change: Fix job editor + browser modal height jumping (stable footer on desktop)

## Why
Switching steps in the job create/edit modal currently changes the modal height (it shrinks/grows based on content up to a max height). This causes the footer action buttons to move vertically, which feels jarring and makes it harder to complete the workflow.

The filesystem path picker (file/directory selection dialog) exhibits similar height changes due to dynamic header/tooling sections (selection summary, validation warnings, async loading), which can also shift the footer.

## What Changes
- Job editor modal:
  - Keep the current modal width.
  - On desktop screens, use a stable modal shell height (no content-driven shrink/grow).
  - Use a `Header / Body / Footer` layout where only the body scrolls.
  - Keep footer actions in a consistent position when switching steps and when validation feedback appears.
- Filesystem / archive browser modals:
  - Apply the same stable shell + scrollable body layout on desktop.
  - Move action buttons into the modal footer and make the content area scrollable so dynamic sections do not push the footer.

## Impact
- Affected specs: `web-ui`
- Affected code (expected):
  - `ui/src/components/jobs/JobEditorModal.vue`
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - (Optional consistency) `ui/src/components/jobs/RunEntriesPickerModal.vue`

## Compatibility / Non-Goals
- No backend API changes.
- No changes to job semantics, validation rules, or persistence.
- No new features; this is a UI layout/UX stability improvement.
