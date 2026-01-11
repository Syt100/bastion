## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for compact table + responsive details
- [x] 1.2 Run `openspec validate update-incomplete-cleanup-table-and-details-ux --strict`

## 2. UI
- [ ] 2.1 Reduce desktop table columns; keep error column compact (type + summary)
- [ ] 2.2 Implement details as desktop modal + mobile bottom drawer
- [ ] 2.3 Add copy actions in details for run/job IDs and last error

## 3. Verification
- [ ] 3.1 Verify visually (desktop + mobile breakpoints)
- [ ] 3.2 Run `cd ui && npm test` and `npm run type-check`
