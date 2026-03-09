# Design: CLI help i18n + reference doc generation

## Context
- Bastion is an OSS-first product, with primary users being individuals/families/small teams.
- The Web UI and in-app docs already share a locale cookie (`bastion_locale`) with default English.
- The CLI is a separate surface: it should default to English but be friendly for Chinese users on Chinese systems.

## Goals / Non-Goals
- Goals:
  - Provide bilingual (`en-US`, `zh-CN`) CLI `--help` output.
  - Use the same translation resource for CLI help and generated docs reference.
  - Make CI fail on missing translations and stale generated docs.
- Non-Goals:
  - Add more locales.
  - Translate all runtime log/error output.
  - Replace docs content translation workflows for prose pages.

## Decisions

### Locale resolution (CLI)
Priority order:
1. `BASTION_LANG` (accepted: `en`, `en-US`, `zh`, `zh-CN`)
2. `LC_ALL`, then `LC_MESSAGES`, then `LANG` (any value indicating `zh*` selects Chinese)
3. Default: `en-US`

### Translation resource
- Store a single Chinese translation file in the Rust crate (JSON map: `key -> translated string`).
- English strings remain the source-of-truth in code (clap derive doc comments).
- Translation keys are stable, derived from the clap command tree:
  - Command about: `<cmd_path>.about`
  - Arg help: `<cmd_path>.arg.<arg_id>.help`
  - Arg long help: `<cmd_path>.arg.<arg_id>.long_help`

Where `<cmd_path>` is like:
- `bastion`
- `bastion.agent`
- `bastion.keypack.export`

### clap help headings
clap's default help renderer has hard-coded English headings (Usage/Options/Commands).
For Chinese output, we provide a custom `help_template` per command that renders:
- usage
- args/flags/options
- subcommands

With Chinese headings, while keeping the actual flag names and env var names unchanged.

### Doc generation (docgen)
- A dedicated `docgen` binary renders help text for the full command tree and writes:
  - `docs/user/reference/cli.generated.md` (English)
  - `docs/zh/user/reference/cli.generated.md` (Chinese)
- The generated content is included by hand-written wrapper pages:
  - `docs/user/reference/cli.md`
  - `docs/zh/user/reference/cli.md`
- In CI, docgen runs in "check" mode:
  - Fails if required translation keys are missing.
  - Fails if generated output differs from committed files (git diff).

## Risks / Trade-offs
- Adding new CLI flags requires adding new translation keys for zh-CN; CI will fail until updated.
  - This is intentional to avoid silently shipping partial Chinese help.
- Help template customization may reduce clap's default formatting flexibility.
  - We keep the template minimal and avoid `{usage-heading}` / `{all-args}` because they emit English headings.
  - Instead, we render our own headings and use `{usage}` / `{options}` / `{positionals}` / `{subcommands}` blocks.
