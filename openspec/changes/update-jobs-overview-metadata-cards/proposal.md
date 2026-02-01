# Change: Enhance Jobs Overview with configuration metadata cards

## Why
The current Jobs Overview focuses on run history and includes a “Quick links” block.
However, operators often need to confirm what a job is configured to do (source/target/format/encryption) at a glance before triggering actions or reviewing failures.

By promoting key configuration metadata into consistent summary cards, we reduce context switching (opening the editor or scanning the Data/Settings pages) and improve mobile ergonomics.

## What Changes
- Remove the Overview “Quick links” block (History/Data shortcuts).
- Add configuration summary cards to Overview:
  - Source type
  - Target type
  - Backup format
  - Encryption
- Present these as compact cards with distinct tags and/or text colors for quick scanning.
- Keep the layout mobile-friendly (cards stack naturally; no extra dedicated row for actions).

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobOverviewSectionView.vue`
  - `ui/src/i18n/locales/*` (new strings)

## Non-Goals
- Changing job behavior, formats, encryption implementation, or backend schemas.
- Adding additional navigation shortcuts beyond the section tabs.
