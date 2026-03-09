## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for responsive layout, mobile card lists, modal sizing, brand icon, and copy punctuation
- [x] 1.2 Run `openspec validate update-web-ui-responsive --strict`

## 2. Web UI
- [x] 2.1 Refactor layout to enforce breakpoint-specific navigation (mobile drawer vs desktop sidebar)
- [x] 2.2 Fix header/content alignment on wide screens (single container baseline)
- [x] 2.3 Replace brand icon with Ionicons `ShieldCheckmark` (solid) and adjust sizing to avoid visual compression
- [x] 2.4 Implement mobile card list mode for Jobs/Agents/Settings (keep DataTable for desktop)
- [x] 2.5 Constrain modal widths on desktop (small/medium/large presets; keep create-job wide)
- [x] 2.6 Standardize subtitles/help copy punctuation (no trailing periods)
- [x] 2.7 Update/extend unit tests for the new responsive/card-list rendering

## 3. Validation
- [x] 3.1 Run `npm test` (ui)
- [x] 3.2 Run `npm run build` (ui)

## 4. Commits
- [x] 4.1 Commit the spec proposal (detailed message)
- [x] 4.2 Commit the UI changes (detailed message with Modules/Tests)
