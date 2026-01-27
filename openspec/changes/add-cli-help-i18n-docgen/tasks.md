## 1. Spec
- [ ] 1.1 Add `cli` spec delta for locale-aware CLI help output and translation key requirements
- [ ] 1.2 Add `dev-workflow` spec delta for docgen + CI enforcement (translation completeness + generated reference pages)
- [ ] 1.3 Run `openspec validate add-cli-help-i18n-docgen --strict`
- [ ] 1.4 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Add CLI locale resolution (`BASTION_LANG` override, auto-detect `zh` from `LC_ALL`/`LC_MESSAGES`/`LANG`, default English)
- [ ] 2.2 Localize clap `Command` (`about`, arg `help`/`long_help`) and apply a Chinese `help_template` (localized section headings)
- [ ] 2.3 Add a shared zh-CN translation map and enforce complete coverage for all required keys
- [ ] 2.4 Add `docgen` binary to generate CLI reference Markdown (English + Chinese) from the same localized command tree
- [ ] 2.5 Add docs wrapper pages and update VitePress sidebar navigation

## 3. Tests / Validation
- [ ] 3.1 Add unit tests for locale resolution and translation completeness checks
- [ ] 3.2 Update CI scripts to run docgen and fail on stale generated docs
- [ ] 3.3 Run `bash scripts/ci.sh`

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
- [ ] 4.2 Mark OpenSpec tasks complete and commit

