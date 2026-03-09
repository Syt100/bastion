## 1. Spec
- [x] 1.1 Add backend spec delta for Dependabot remediation policy
- [x] 1.2 Run `openspec validate update-dependabot-alert-remediation --strict`

## 2. Rust Alerts Remediation
- [x] 2.1 Reduce SQLx unused dependency surface (disable default features)
- [x] 2.2 Update lockfile to patched Rust versions for `time` and `bytes`
- [x] 2.3 Verify `rsa` alert is resolved (upgrade or graph removal)

## 3. npm Alerts Remediation
- [x] 3.1 Add UI overrides and regenerate lockfile for patched `lodash`/`lodash-es`
- [x] 3.2 Attempt docs `esbuild` patched override and keep only if compatible

## 4. Validation
- [x] 4.1 Run `cargo fmt --all`
- [x] 4.2 Run targeted Rust tests for touched crates
- [x] 4.3 Run `bash scripts/ci.sh`
- [x] 4.4 Verify GitHub Actions + Dependabot alert status after push
- [x] 4.5 Mark checklist complete
