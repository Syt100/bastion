## Why
Current GitHub Release notes are generated from raw commit history, which frequently includes internal-only commits (for example `chore(spec)`), making release notes noisy for end users.

We need a single, curated changelog source focused on user-visible changes.

## What Changes
- Add a root `CHANGELOG.md` using Keep a Changelog style sections (`Unreleased` + versioned entries).
- Add `scripts/changelog.sh` to validate changelog structure and extract a version section by tag for release publishing.
- Update `.github/workflows/release.yml` to publish GitHub Release notes from `CHANGELOG.md` instead of raw `git log`.
- Add changelog checks to CI and document contributor expectations in `README.md`.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `CHANGELOG.md`, `.github/workflows/release.yml`, `scripts/changelog.sh`, `scripts/changelog_test.sh`, `scripts/ci.sh`, `README.md`
