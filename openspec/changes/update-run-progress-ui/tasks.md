## 1. Spec
- [x] 1.1 Add backend spec delta for enriched backup progress snapshots (source vs transfer totals) and raw_tree_v1 stable upload totals
- [x] 1.2 Add web-ui spec delta for Run Detail Progress panel redesign (overall bar + stage breakdown + mobile)
- [x] 1.3 Add design.md decisions for progress semantics + UI layout
- [x] 1.4 Run `openspec validate update-run-progress-ui --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Backend: Progress Semantics
- [ ] 2.1 Propagate filesystem build stats (raw-tree data bytes/files) and stable source totals from packaging
- [ ] 2.2 Publish stable transfer totals during upload (raw_tree_v1 and archive_v1) and store in progress snapshot detail
- [ ] 2.3 Keep source totals visible across stage transitions via snapshot detail
- [ ] 2.4 Add/update unit tests for snapshot detail behavior (where applicable)

## 3. Web UI: Progress Panel
- [ ] 3.1 Add a Progress panel component for Run Detail (overall bar + stage stepper + key stats)
- [ ] 3.2 Add per-stage help ("?") tooltips explaining scan/packaging/upload (raw_tree_v1 specifics included)
- [ ] 3.3 Mobile layout: stacked + collapsible sections; preserve readability on small screens
- [ ] 3.4 Add unit tests for progress rendering (percent/indeterminate + stage breakdown)

## 4. Validation
- [ ] 4.1 Run `./scripts/ci.sh`

## 5. Commits
- [ ] 5.1 Commit backend changes (detailed message with Modules/Tests)
- [ ] 5.2 Commit web-ui changes (detailed message with Components/Tests)
