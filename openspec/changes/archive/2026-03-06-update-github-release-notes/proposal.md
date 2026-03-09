## Why
The first GitHub Release was created successfully but the auto-generated release notes were nearly empty.
We want each release page to include a usable change log without maintaining a separate `CHANGELOG.md`.

## What Changes
- Update the release workflow to generate a changelog section from git history between tags and use it as the GitHub Release body.

## Impact
- No runtime behavior change.
- Release pages become more informative even when releases are created without PR-based workflows.
