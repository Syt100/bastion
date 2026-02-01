## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (routes, layout, mobile/desktop behavior)
- [x] 1.3 Add `web-ui` spec delta (workspace + drawer + section model)
- [x] 1.4 Run `openspec validate refactor-jobs-workspace-master-detail --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Replace Jobs routing with Jobs workspace route family:
  - `/n/:nodeId/jobs`
  - `/n/:nodeId/jobs/:jobId/(overview|history|data)`
  - `/n/:nodeId/jobs/:jobId/(overview|history|data)/runs/:runId`
- [x] 2.2 Implement JobsWorkspaceShell (desktop master-detail; mobile single-column)
- [x] 2.3 Implement JobWorkspaceView (stable header + section tabs + router-view)
- [x] 2.4 Implement sections:
  - Overview (health + key config + primary actions)
  - History (runs list)
  - Data (snapshots + retention in one page)
- [x] 2.5 Implement RunDetailOverlay:
  - desktop side drawer
  - mobile full-screen drawer
  - close drawer via navigation back to parent section route
- [x] 2.6 Refactor Run Detail into an embeddable panel component used by the drawer overlay
- [x] 2.7 Move JSON/inspect into a job “More” menu (remove it as a top-level section)
- [x] 2.8 Update all links that open runs to use job-scoped run overlay routes (Dashboard, Snapshots, etc.)
- [x] 2.9 Remove old Jobs/Job Detail/Run Detail routes and views that are no longer used

## 3. Tests / Validation
- [x] 3.1 Add/update unit tests for the new routing structure (desktop + mobile semantics)
- [x] 3.2 Add tests for run drawer open/close navigation behavior
- [x] 3.3 Run `npm test --prefix ui`
- [x] 3.4 Run `bash scripts/ci.sh`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
