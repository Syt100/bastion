## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for preferred node + node-scoped header cues
- [x] 1.2 Run `openspec validate update-node-context-navigation --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [ ] 2.1 Add preferred node state to UI store (persisted)
- [ ] 2.2 Update AppShell node selector behavior (route-scoped vs global)
- [ ] 2.3 Update Jobs/Storage navigation to use preferred node when needed
- [ ] 2.4 Add node context cue to node-scoped page headers (Jobs, Job detail, Snapshots, Run detail)

## 3. Tests / Validation
- [ ] 3.1 Add/update unit tests for preferred node navigation behavior
- [ ] 3.2 Run `npm test --prefix ui`

## 4. Commits
- [ ] 4.1 Commit implementation changes (detailed message)
