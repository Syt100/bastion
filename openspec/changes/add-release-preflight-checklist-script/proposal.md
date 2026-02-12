## Why
Release checks are currently split across multiple commands, making it easy to miss a step when preparing a tag.

We need one repeatable preflight command for maintainers and to keep release workflow behavior aligned with local release practice.

## What Changes
- Add `scripts/release-preflight.sh` to run release checks for a target tag.
- Add `scripts/release-preflight_test.sh` for regression coverage.
- Update `.github/workflows/release.yml` to call the preflight script for release notes generation.
- Update docs and skill references with the preflight command.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `scripts/release-preflight.sh`, `scripts/release-preflight_test.sh`, `.github/workflows/release.yml`, `README.md`, `skills/maintain-changelog-release/references/release-playbook.md`
