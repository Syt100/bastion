## 1. Implementation
- [ ] 1.1 Derive stage start/end timestamps from run events and/or progress snapshots
- [ ] 1.2 Display per-stage durations (Scan/Build/Upload) and total duration in Run Progress panel
- [ ] 1.3 Display final transfer speed after completion (average over upload stage; fallback to overall)
- [ ] 1.4 Display peak transfer speed during the run when snapshot includes rate information
- [ ] 1.5 Indicate the failure stage when a run ends in failed/rejected (when determinable)
- [ ] 1.6 Add/adjust unit tests for timeline/metrics calculations

## 2. Validation
- [ ] 2.1 `npm -C ui run type-check`
- [ ] 2.2 `npm -C ui run test:unit`
