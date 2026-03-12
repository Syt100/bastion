## 1. Spec
- [x] 1.1 Add `cli` spec delta for `bastion config` and `bastion doctor` (outputs + exit codes + locale behavior)
- [x] 1.2 Add `dev-workflow` spec delta for generated config/env reference pages and CI drift checks
- [x] 1.3 Run `openspec validate add-cli-doctor-and-config-reference --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Add `config` and `doctor` subcommands and wire them into main
- [x] 2.2 Implement `bastion config` (text + `--json`) including sources matching runtime config behavior
- [x] 2.3 Implement `bastion doctor` checks (text + `--json`) with actionable messages and non-zero exit on failures
- [x] 2.4 Update CLI help translations (zh-CN) for new commands and flags (CI must fail on missing keys)
- [x] 2.5 Extend docgen to generate config/env reference docs (en/zh) and add VitePress navigation

## 3. Tests / Validation
- [x] 3.1 Add unit tests for config/source resolution helpers and doctor checks that are easy to regress
- [x] 3.2 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
- [x] 4.2 Mark OpenSpec tasks complete and commit
