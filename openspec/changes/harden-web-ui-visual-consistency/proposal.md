# Change: Harden Web UI Visual Consistency (Tokens, Recipes, Guardrails, and Style Guide)

## Why
The Web UI already has a strong foundation (shared `--app-*` tokens + Naive UI theme overrides), but day-to-day iteration still introduces inconsistency:
- ad-hoc Tailwind colors (`text-red-*`, `bg-white/..`, `border-black/..`) bypass theme tokens and drift between light/dark + presets,
- “muted” text relies on `opacity-*`, producing uneven contrast across themes and backgrounds,
- repeated UI patterns (cards, list rows, dividers, toolbars, tags) are implemented slightly differently per page,
- there is no single, authoritative place to learn the intended visual rules, so consistency depends on reviewer memory.

We want a durable, scalable “design system layer” that makes the consistent choice the easy choice:
- document the rules,
- encode the rules into tokens + reusable recipes,
- and add lightweight guardrails so regressions are caught early.

## What Changes
- Expand the Web UI token set with a small number of globally stable “foundation” tokens (radii / motion / optional typography helpers) and standard utility classes for common semantics (muted text, divider/border, inset surface).
- Standardize key component recipes (Card, ListRow, Toolbar, Tag/Badge, Table, EmptyState) and refactor existing screens to use them.
- Remove remaining hard-coded Tailwind “visual” colors and replace them with token-driven equivalents (prefer `var(--app-...)` or Naive UI semantic components).
- Add a maintainers-facing UI style guide in the docs site (English + zh-CN) that defines tokens, allowed scales, component recipes, and do/don’t examples.
- Add a lightweight repository check to prevent reintroducing banned “non-token” patterns in `ui/src` (fast ripgrep-based guardrail).

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/styles/main.css`, shared UI components, several views, docs site navigation/content, and CI scripts

