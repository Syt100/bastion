# Change: Update Settings Navigation (Sidebar Submenu + Settings Overview + Mobile List-First)

## Why
The current Settings UX introduces a second navigation layer inside the Settings page (a submenu within Settings).
On desktop, this feels redundant because a persistent global sidebar already exists.
On mobile, a select/tabs-based navigation is less “app-like” than a list-first Settings flow.

This change aims to:
- Reduce navigation duplication on desktop by moving Settings sub-navigation into the global sidebar.
- Provide a dedicated Settings overview page for both desktop and mobile.
- Use a mobile-friendly list-first navigation for Settings and Notifications.

## What Changes
- Desktop sidebar:
  - `Settings` becomes a parent menu that only expands/collapses (no route navigation).
  - Add child entries: `Overview (/settings)`, `Storage (/settings/storage)`, `Notifications (/settings/notifications)`.
  - When the current route is under `/settings/**`, the Settings submenu SHOULD be expanded and the correct child SHOULD be highlighted.
- Settings page:
  - `/settings` becomes an Overview page that lists settings areas (Storage, Notifications, ...).
  - The Settings page MUST NOT show a second persistent submenu on desktop (avoid “menu within menu” duplication).
- Notifications page:
  - `/settings/notifications` becomes an index/list page (Channels, Destinations, Templates, Queue).
  - On mobile, navigation MUST be list-first (“enter list, then enter subpage”), not tabs/select.
  - On desktop, Notifications subpages MAY still provide a lightweight in-module navigation (tabs) while remaining route-based.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` router, layout sidebar menu, Settings/Notifications views, i18n strings, and UI unit tests

