## Why

Security updates currently bypass the version-update groups and create many individual pull requests, including follow-up upgrades immediately after older updates merge.

## What Changes

- Add explicit security-update groups for Cargo, UI npm, Docs npm, and GitHub Actions.
- Group minor and patch security updates within each existing ecosystem/directory entry; leave major security updates individually reviewable.
- Keep existing schedules, version-update groups, and PR limits.

## Capabilities

### Modified Capabilities
- `ci`: dependency security-update grouping.

## Impact

Only `.github/dependabot.yml` changes behavior. No application dependencies or runtime code change. Dependabot applies the rules after merge; existing PRs may need a later refresh and are not closed by this change.
