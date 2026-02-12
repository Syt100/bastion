# Changelog Writing Guide

## What to include

- Include end-user visible changes.
- Include operationally relevant behavior changes.
- Include security fixes or compatibility notes.

## What to skip

- Skip internal-only chores (`chore(spec)`, pure CI maintenance).
- Skip implementation-only refactors without user impact.
- Skip test-only additions unless they change user behavior guarantees.

## Category mapping

- `Added`: new feature/capability.
- `Changed`: behavior/UX/default flow change.
- `Deprecated`: still available, scheduled for removal.
- `Removed`: removed behavior/API.
- `Fixed`: bug fix affecting users.
- `Security`: security hardening with user/operator relevance.

## Entry style

- Write one outcome per bullet.
- Start with a verb in past tense or imperative style.
- Mention scope (UI/API/CLI/ops) when helpful.
- Keep implementation details minimal.

## Examples

Good:
- `- Added bulk retry action for failed operations in Jobs UI.`
- `- Fixed setup flow to show weak-password minimum length error correctly.`
- `- Changed release notes generation to use curated CHANGELOG sections.`

Avoid:
- `- chore(spec): update tasks`
- `- refactor: split module`
- `- fix lint`
