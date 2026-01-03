# Change: Refactor Web UI Style System (Shared Tokens, Remove Legacy Views)

## Why
As the Web UI grows, keeping visual consistency becomes harder when:
- pages duplicate long Tailwind class strings for common patterns (glass surfaces, list rows, helper text, icon tiles),
- legacy/unused view files remain in the codebase and can be accidentally modified or reintroduced.

We want a small, reusable style “kit” that makes future UI work faster and safer, while reducing dead code.

## What Changes
- Add a set of reusable UI utility classes for common UI patterns:
  - glass surfaces used in navigation chrome
  - list row/hover styles for “settings-like” lists
  - muted helper text style
  - icon tile style for list items
- Apply these shared utilities in places where the pattern repeats.
- Remove unused legacy view files that are not referenced by the router/tests.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` styles and view/layout files

