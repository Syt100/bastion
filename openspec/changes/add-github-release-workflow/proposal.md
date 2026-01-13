## Why
Bastion is being open-sourced, and we need a repeatable way to publish downloadable binaries for end users.

## What Changes
- Add a GitHub Actions release workflow that builds and publishes binaries for:
  - Linux x64
  - Windows x64

## Impact
- No runtime behavior change.
- Adds a release automation workflow under `.github/workflows/`.
