# Change: Improve list-page clarity, accessibility, and interaction feedback

## Why
The previous list-page refactor solved structural consistency, but several UX gaps remain in day-to-day use:

- Jobs result metrics are ambiguous and currently do not clearly distinguish visible items from total matches.
- Jobs mobile filters lack persistent active-condition visibility.
- Jobs rows still mix row-level activation with nested controls in a way that is not ideal for accessibility semantics.
- Notifications Queue lacks explicit loading-empty/no-data/no-results state guidance.
- Pagination controls are shared, but range context (for example, "1-20 of 238") is not consistently surfaced across Jobs/Agents/Notifications.
- Agents mobile cards remain dense, with secondary information competing with primary scan targets.
- Some high-frequency row actions (for example, "Run now") need stronger in-flight feedback to prevent duplicate operations.
- Search-triggered list refresh cadence should be standardized (debounced) across list pages.

## What Changes
- Clarify Jobs list metrics with explicit visible-count vs filtered-total labeling.
- Add active-filter chips visibility in Jobs mobile list context, including clear behavior parity.
- Refactor Jobs list-row activation semantics so row navigation/select triggers are explicit and action controls remain isolated.
- Add Notifications Queue empty-state variants for loading-empty, no-data, and filtered-no-results states.
- Standardize pagination summary labels with visible range + total count across Jobs, Agents, and Notifications Queue.
- Reduce Agents mobile card noise by prioritizing primary fields and moving secondary fields into progressive disclosure.
- Add consistent per-row in-flight feedback and temporary action disabling for key async actions.
- Standardize search-triggered refresh debounce timing across list pages.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.spec.ts`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/AgentsView.spec.ts`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.spec.ts`
  - `ui/src/i18n/locales/en-US.ts`
  - `ui/src/i18n/locales/zh-CN.ts`
  - shared list helper(s) under `ui/src/lib/`
- Backend APIs: no changes.

## Non-Goals
- Introducing virtual scrolling in this change.
- Changing server-side pagination/filter contracts.
- Reworking non-list pages or global shell layout.
