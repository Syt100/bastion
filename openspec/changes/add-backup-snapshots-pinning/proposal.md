# Change: Add Snapshot Pinning (Protect from Retention / Safer Deletion)

## Why
Users often need to protect certain backup points from automated cleanup (e.g., "last known good" or "monthly archive").

We need a first-class "pin/protect" flag that:
- excludes snapshots from retention selection
- adds stronger guardrails for manual deletion

High-level design reference: `docs/backup-snapshots.md`.

## What Changes
- Add `pinned_at` / `pinned_by_user_id` fields to the snapshot index (`run_artifacts`).
- Add APIs to pin/unpin a snapshot.
- Update snapshot deletion to prevent accidental deletion of pinned snapshots (requires explicit force/extra confirmation).
- Update the snapshots UI page to show pin state and provide pin/unpin actions.

## Scope
- Pin/unpin semantics in the snapshot index.
- Guardrails for manual deletion.

## Out of Scope (Follow-ups)
- Advanced retention rule sets (GFS).
- RBAC/permissions beyond the current single-user model.

## Key Decisions
- Pin is stored in Hub DB only and applies regardless of target type.
- Retention logic will treat pinned snapshots as non-deletable by policy.

## Risks
- Without retention implemented yet, pin only affects manual delete initially; ensure semantics remain consistent once retention is added.

## Success Criteria
- Users can pin/unpin snapshots in the UI.
- Pinned snapshots clearly display as protected and cannot be deleted accidentally.

