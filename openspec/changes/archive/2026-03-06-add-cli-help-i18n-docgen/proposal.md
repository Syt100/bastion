# Change: Add CLI --help i18n and auto-generate CLI reference docs

## Why
Today Bastion's CLI help output and the product docs can drift:

- `bastion --help` is English-only and does not respect the user's locale.
- Documentation pages that describe CLI flags / environment variables are easy to forget to update and become outdated.
- Maintaining separate translations for CLI help and docs reference is error-prone.

For an OSS-first product with a small target scale, we want:

- A predictable default (**English**), with automatic Simplified Chinese output when the system locale indicates `zh`.
- A single translation resource for CLI help + generated docs reference.
- CI enforcement so missing translations and stale generated reference docs fail fast.

## What Changes
- CLI help i18n:
  - Add locale resolution for CLI help (default English; auto-detect `zh` from `LC_ALL`/`LC_MESSAGES`/`LANG`; allow override via `BASTION_LANG`).
  - Localize `about` and all arg help/long-help strings for `--help` output.
  - Provide a Chinese help template so section headings (Usage/Options/Commands) are also Chinese.
- Docs reference generation:
  - Add a Rust `docgen` binary that renders the CLI help output (English + Chinese) into generated Markdown under `docs/`.
  - Reuse the same Chinese translation map as the CLI help i18n.
  - Fail generation if any required Chinese translation key is missing.
- CI enforcement:
  - Run docgen in `scripts/ci.sh` and `scripts/ci.ps1`.
  - Fail CI when generated docs differ from the committed output.
- Docs navigation:
  - Add "Reference/参考" entries in the VitePress sidebar for the generated CLI reference page.

## Impact
- Affected specs: `cli`, `dev-workflow`
- Affected code:
  - `crates/bastion/src/*` (CLI wiring + i18n + docgen)
  - `docs/*` (new reference pages + sidebar)
  - `scripts/ci.sh`, `scripts/ci.ps1`

## Non-Goals
- Adding more locales beyond `en-US` and `zh-CN`.
- Translating runtime error messages; this change targets CLI `--help` and generated reference docs only.
- A full "configuration schema" reference; this change starts by auto-generating a CLI reference page.

