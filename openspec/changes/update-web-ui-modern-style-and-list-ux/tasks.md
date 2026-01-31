## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for: modern colorful theme, surface hierarchy, navigation chrome refresh, action hierarchy, ListToolbar/SelectionToolbar, dashboard health summary, and node-context clarity
- [x] 1.2 Run `openspec validate update-web-ui-modern-style-and-list-ux --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Visual Foundations (Tokens + Chrome)
- [x] 2.1 Introduce design tokens (CSS variables) for background/surfaces/text/accents/status colors (light + dark)
- [x] 2.2 Map Naive UI theme overrides to the tokens (primary + status + surfaces) for consistent component colors
- [x] 2.3 Refresh shared surface utilities (`app-card`, `app-glass`, `app-icon-tile`, tables/list rows) to reduce "industrial" weight
- [x] 2.4 Refresh AppShell chrome (sider/header/menu active states) to match the new modern style
- [x] 2.5 Commit visual foundations (detailed message)

## 3. Web UI - Shared List UX Components
- [x] 3.1 Add a shared `ListToolbar` component (search/filters/sort/view toggle/actions; responsive layout)
- [x] 3.2 Add a shared `SelectionToolbar` component (selected count + bulk actions + scope hint)
- [x] 3.3 Add a shared overflow actions pattern for secondary/danger actions
- [x] 3.4 Commit shared list UX components (detailed message)

## 4. Web UI - Apply List UX + Action Hierarchy
- [x] 4.1 Agents: add search + status quick filters; move most row actions into overflow; make dangerous actions consistently confirmed
- [ ] 4.2 Jobs: add search/filter/sort; standardize primary vs secondary actions; make row click open detail (desktop)
- [ ] 4.3 Snapshots: cursor pagination ("Load more"); add filters (status/pinned/target); improve selection + bulk delete flow
- [ ] 4.4 Align Bulk Operations / Notifications queue / Maintenance cleanup toolbars to the shared ListToolbar visuals
- [ ] 4.5 Commit list UX + action hierarchy updates (detailed message)

## 5. Web UI - Dashboard as Status Center
- [ ] 5.1 Add top-level health summary cards with actionable links
- [ ] 5.2 Improve empty-state CTAs for common first-run flows (Agents/Jobs)
- [ ] 5.3 Commit dashboard status center updates (detailed message)

## 6. Web UI - Node Context Clarity
- [ ] 6.1 Clarify node picker semantics (current view vs preference) and improve node-scoped page indicators
- [ ] 6.2 Commit node context clarity updates (detailed message)

## 7. Validation
- [ ] 7.1 Run `npm test --prefix ui`
- [ ] 7.2 Run `npm run lint --prefix ui`
- [ ] 7.3 Run `npm run build --prefix ui`
- [ ] 7.4 Run `bash scripts/ci.sh`
