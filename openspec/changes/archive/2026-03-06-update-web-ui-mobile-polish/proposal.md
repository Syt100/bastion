# Change: Update Web UI Mobile Polish (Breakpoints, Header Menu, Wizard Steps)

## Why
After improving the Web UI layout, a few usability and maintainability issues remain:
- Responsive breakpoint logic uses repeated hard-coded media queries in multiple files.
- The console shows `[Vue Router warn]: No match found for location with path "/undefined"` after login.
- On mobile, top-bar actions (language/theme/logout) can overflow the header.
- On mobile, the Jobs create/edit wizard stepper can overflow horizontally.
- The "MVP" tag should be updated to "Beta" (test version) across desktop and mobile.

## What Changes
- Centralize responsive breakpoints and shared UI constants (avoid scattered hard-coded values).
- Guard menu navigation so invalid/undefined keys never trigger router navigation.
- Replace mobile header actions with a compact “More” dropdown menu.
- Make the Jobs wizard step indicator mobile-friendly (step x/total + progress bar).
- Replace "MVP" label with "Beta" in the UI.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` layout and Jobs view, i18n strings

