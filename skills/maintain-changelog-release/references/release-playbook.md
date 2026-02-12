# Release Playbook

## Pre-release preparation

1. Confirm target version tag: `vX.Y.Z`.
2. Ensure `## [Unreleased]` contains curated user-facing items.
3. Move finalized items into `## [vX.Y.Z] - YYYY-MM-DD`.
4. Reset `Unreleased` categories for future work.

## Required validation

Run:

```bash
bash scripts/release-preflight.sh --tag vX.Y.Z --output release-notes.md
```

Confirm `release-notes.md` starts with:

```md
## [vX.Y.Z] - YYYY-MM-DD
```

## Git steps

```bash
git add CHANGELOG.md
git commit -m "release: prepare changelog for vX.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

## Failure handling

- If `check` fails, fix changelog structure/categories first.
- If `extract` fails, add/fix the matching version header in `CHANGELOG.md`.
- Do not push tag until both checks pass.
