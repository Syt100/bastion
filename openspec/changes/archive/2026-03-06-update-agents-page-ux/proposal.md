# Change: Improve Agents page UX (quick links, enrollment command template)

## Why
The Agents page is a primary operational surface: users need to quickly jump from an agent to the agent’s Jobs/Storage context and enroll new agents reliably.

Today:
- navigation from an agent to node-scoped pages is not one-click
- enrollment token UX requires users to manually translate the token into a correct CLI command

## What Changes
- Add quick navigation actions per agent:
  - Open that agent’s Jobs (`/n/:agentId/jobs`)
  - Open that agent’s Storage settings (`/n/:agentId/settings/storage`)
- Improve the enrollment token modal:
  - Show a copyable “enroll command template” that includes the Hub URL and the generated token

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/AgentsView.vue`
  - `ui/src/i18n/locales/*`

## Non-Goals
- Agent auto-upgrade / lifecycle management.
- Backend changes.

