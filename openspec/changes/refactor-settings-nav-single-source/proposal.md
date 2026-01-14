## Why
We added a new Settings page (About) and it appeared in the Settings overview list but was missing from the Settings sidebar submenu.
This happened because Settings navigation items were duplicated across multiple components.

## What Changes
- Introduce a single Settings navigation config as the source of truth.
- Derive both Settings overview cards and the desktop Settings submenu from the same config.
- Add unit tests to prevent items from appearing in overview but missing in submenu.

## Impact
- No behavior change beyond making Settings navigation consistent.
- Improves maintainability when adding new Settings sections.
