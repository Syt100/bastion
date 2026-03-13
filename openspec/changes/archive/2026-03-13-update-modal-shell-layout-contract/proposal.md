# Change: Update modal shell layout contract

## Why
Recent modal migrations moved height constraints into `content-style` in several dialogs. This made container-level size limits inconsistent, and long forms can grow beyond the intended viewport bounds.

## What Changes
- Define a clear `AppModalShell` layout contract that separates container sizing from body/content sizing.
- Ensure `scrollBody=false` mode still provides bounded height + overflow behavior for long content.
- Migrate task create/edit dialog sizing to the new contract so height limits are applied at the correct layer.
- Add unit tests for modal shell layout contract and job editor dialog height-bound behavior.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/AppModalShell.vue`
  - `ui/src/styles/main.css`
  - `ui/src/components/jobs/JobEditorModal.vue`
  - `ui/src/components/AppModalShell.spec.ts`
  - `ui/src/components/jobs/JobEditorModal.spec.ts`

## Non-Goals
- Changing create/edit task workflow, field validations, or submit payload semantics.
- Visual redesign of modal header/footer styles beyond layout contract fixes.
