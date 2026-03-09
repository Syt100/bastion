# Change: Open-source repository on GitHub (Apache-2.0 + CI)

## Why
We want to publish this project as an open-source repository on GitHub with a clear license and baseline automation that prevents common regressions and accidental secret leaks.

## What Changes
- Add Apache-2.0 licensing metadata (`LICENSE`) for open-source distribution.
- Add GitHub Actions CI to run the repo CI script (`bash scripts/ci.sh`), which includes:
  - Rust fmt/clippy/test
  - UI tests
  - gitleaks secret scanning
- Document the license/CI entry point in the root README.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `LICENSE`, `.github/workflows/ci.yml`, `README.md`

## Non-Goals
- Publishing releases/packages.
- Adding pre-commit hooks.

