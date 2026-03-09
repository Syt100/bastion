## 1. Spec
- [x] 1.1 Add `cli` spec delta for locale-aware CLI help output and translation key requirements
- [x] 1.2 Add `dev-workflow` spec delta for docgen + CI enforcement (translation completeness + generated reference pages)
- [x] 1.3 Run `openspec validate add-cli-help-i18n-docgen --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add CLI locale resolution (`BASTION_LANG` override, auto-detect `zh` from `LC_ALL`/`LC_MESSAGES`/`LANG`, default English)
- [x] 2.2 Localize clap `Command` (`about`, arg `help`/`long_help`) and apply a Chinese `help_template` (localized section headings)
- [x] 2.3 Add a shared zh-CN translation map and enforce complete coverage for all required keys
- [x] 2.4 Add `docgen` binary to generate CLI reference Markdown (English + Chinese) from the same localized command tree
- [x] 2.5 Add docs wrapper pages and update VitePress sidebar navigation

## 3. Tests / Validation
- [x] 3.1 Add unit tests for locale resolution and translation completeness checks
- [x] 3.2 Update CI scripts to run docgen and fail on stale generated docs
- [x] 3.3 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [x] 4.2 Mark OpenSpec tasks complete and commit
