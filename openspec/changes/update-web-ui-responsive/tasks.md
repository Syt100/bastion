## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for responsive layout, mobile card lists, modal sizing, brand icon, and copy punctuation
- [ ] 1.2 Run `openspec validate update-web-ui-responsive --strict`

## 2. Web UI
- [ ] 2.1 Refactor layout to enforce breakpoint-specific navigation (mobile drawer vs desktop sidebar)
- [ ] 2.2 Fix header/content alignment on wide screens (single container baseline)
- [ ] 2.3 Replace brand icon with Ionicons `ShieldCheckmark` (solid) and adjust sizing to avoid visual compression
- [ ] 2.4 Implement mobile card list mode for Jobs/Agents/Settings (keep DataTable for desktop)
- [ ] 2.5 Constrain modal widths on desktop (small/medium/large presets; keep create-job wide)
- [ ] 2.6 Standardize subtitles/help copy punctuation (no trailing periods)
- [ ] 2.7 Update/extend unit tests for the new responsive/card-list rendering

## 3. Validation
- [ ] 3.1 Run `npm test` (ui)
- [ ] 3.2 Run `npm run build` (ui)

## 4. Commits
- [ ] 4.1 Commit the spec proposal (detailed message)
- [ ] 4.2 Commit the UI changes (detailed message with Modules/Tests)

