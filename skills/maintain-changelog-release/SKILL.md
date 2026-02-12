---
name: maintain-changelog-release
description: Curate Bastion changelog entries and execute release-note preparation from `CHANGELOG.md`. Use when asked to add/update `## [Unreleased]` items, create a versioned changelog section (`## [vX.Y.Z] - YYYY-MM-DD`), validate changelog format, or prepare tag-based release notes for GitHub Release publishing.
---

# Maintain Changelog Release

## Overview

Keep `CHANGELOG.md` user-facing, concise, and release-ready.  
Drive release notes from the versioned changelog section instead of raw commit history.

## Core Rules

- Treat `CHANGELOG.md` as the single source of truth for release notes.
- Write only user-visible impact (behavior, UX, compatibility, security, operations).
- Omit internal-only noise (`chore(spec)`, CI-only churn, refactor-only internals without user impact).
- Use only these category headers: `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.
- Keep entries short, concrete, and outcome-focused.

See detailed wording patterns in `references/changelog-writing.md`.

## Workflow 1: Update Unreleased

1. Review merged or pending changes with user impact.
2. Add bullets under `## [Unreleased]` in the correct category.
3. Remove placeholder lines (for example `_No user-facing changes yet._`) from categories that now have real entries.
4. Keep unaffected categories unchanged.
5. Run:
   ```bash
   bash scripts/changelog.sh check
   ```

## Workflow 2: Cut A Release Section

1. Choose release tag and date (`vX.Y.Z`, `YYYY-MM-DD`).
2. Move finalized entries from `## [Unreleased]` into:
   ```md
   ## [vX.Y.Z] - YYYY-MM-DD
   ```
3. Recreate/retain `Unreleased` categories for future work.
4. Validate structure:
   ```bash
   bash scripts/changelog.sh check
   ```
5. Verify release-note extraction matches the target tag:
   ```bash
   bash scripts/changelog.sh extract --tag vX.Y.Z --output release-notes.md
   ```

See the full checklist in `references/release-playbook.md`.

## Workflow 3: Publish Release (If User Asks To Execute)

1. Ensure release section exists in `CHANGELOG.md` for the exact tag.
2. Run changelog checks (and optionally full CI when requested):
   ```bash
   bash scripts/changelog.sh check
   bash scripts/changelog_test.sh
   ```
3. Commit changelog changes.
4. Create and push tag:
   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```
5. Confirm GitHub Actions release workflow consumes the extracted section.

## Output Expectations

- When asked to draft changelog text, produce ready-to-paste markdown grouped by allowed categories.
- When asked to execute, run validation commands and report exact pass/fail outcomes.
- When tag/section mismatches exist, stop and fix `CHANGELOG.md` before release steps.

## References

- `references/changelog-writing.md`: entry-writing rubric and examples.
- `references/release-playbook.md`: release cut and tag checklist.
