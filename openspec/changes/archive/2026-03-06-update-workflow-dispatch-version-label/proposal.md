## Why
Manual `workflow_dispatch` release builds currently use placeholder versioning (`v0.0.0`) and branch-name-based asset labels. This makes build metadata hard to trace and does not reflect the baseline release lineage.

We want manual builds to carry a deterministic preview version derived from the latest release tag plus the current commit short hash.

## What Changes
- Update release workflow version resolution logic.
- Keep tag-triggered behavior unchanged.
- For `workflow_dispatch`, compute a manual display version as `<latest-tag-without-v>-dh<short-hash>`.
- Use this computed version for build metadata and asset naming in manual builds.
- Keep package semantic version fields numeric by deriving from the latest tag.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `.github/workflows/release.yml`, `CHANGELOG.md`
