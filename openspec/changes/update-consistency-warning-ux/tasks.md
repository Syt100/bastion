## 1. Specification
- [ ] Write spec deltas for `control-plane`, `web-ui`, `notifications`
- [ ] `openspec validate update-consistency-warning-ux --strict`

## 2. Implementation (in order)

### 2.1 Control-plane: runs list early warning
- [ ] Update `GET /api/jobs/:id/runs` to derive `consistency_changed_total` from the latest `source_consistency` event when `summary_json` is absent
- [ ] Add/adjust HTTP tests for the running-run case and precedence rules

### 2.2 Web UI: run detail consistency section
- [ ] Extend `ui/src/lib/run_summary.ts` to parse consistency breakdown + sample (not just total)
- [ ] Render a Consistency section in run detail (breakdown + sample + truncation)
- [ ] Add “view consistency events” one-click action (switch to Events tab + set kind filter)
- [ ] Add i18n strings (`zh-CN`, `en-US`)
- [ ] Add UI unit tests

### 2.3 Notifications
- [ ] Include consistency warning count in notification templates (email/wecom) when present
- [ ] Add tests for template rendering

## 3. Validation
- [ ] Run `scripts/ci.sh`

