## Context
The Web UI already uses a themeable token system (`--app-*`) and derives Naive UI `themeOverrides` from computed token values. This gives us cross-theme consistency *when tokens are used*.

In practice, several pages/components still use “direct styling” Tailwind classes (hard-coded colors, opacity-based muted text, dark/light split borders) that bypass tokens and create drift.

This change hardens the design system by:
- expanding the *foundation* layer (non-theme, globally stable tokens),
- defining a small set of reusable “recipes” (patterns that are repeated across the UI),
- and adding guardrails to prevent regressions.

## Goals / Non-Goals

### Goals
- Make UI consistency *default* by minimizing ad-hoc styling choices in views.
- Ensure muted text, dividers, borders, and inset surfaces are token-driven and render consistently across light/dark + theme presets.
- Reduce the number of “one-off” arbitrary Tailwind values (e.g. `text-[17px]`, `bg-white/40`) and replace them with approved scales.
- Provide a developer-facing style guide (EN + zh-CN) that documents tokens, allowed scales, and component recipes with do/don’t examples.
- Add a fast guardrail check that detects reintroduced non-token patterns in `ui/src`.

### Non-Goals
- No redesign of information architecture or workflows.
- No pixel-perfect overhaul of every screen in a single pass; focus on high-leverage shared components and repeated patterns first.
- No introduction of a full design-token build pipeline (e.g. Style Dictionary). Keep it lightweight and repo-local.

## Token Strategy

### Two Layers: Theme Tokens vs Foundation Tokens
- **Theme tokens** (per preset; light + dark) define brand, surfaces, text, borders, and state colors.
  - Examples: `--app-primary`, `--app-text`, `--app-border`, `--app-surface-2`.
- **Foundation tokens** (global; not per theme) define stable system primitives:
  - radii, motion durations/easing, and any cross-theme typography helpers.

Rationale: theme presets should not “randomly” change spacing/radius/motion; these are product-level consistency choices.

### Tailwind Usage Rules (Long-Term)
Prefer Tailwind’s built-in scales for spacing/typography/radius **without** arbitrary values:
- Spacing: use the Tailwind scale (4px base).
- Radius: standardize on `rounded-lg` (8px), `rounded-xl` (12px), `rounded-2xl` (16px).
- Typography: standardize on `text-xs/sm/base/lg/xl/2xl` (no `text-[Npx]` unless justified).

When Tailwind needs a color, it MUST be token-driven:
- Allowed: `bg-[var(--app-surface-2)]`, `border-[var(--app-border)]`, `text-[var(--app-text-muted)]`, `ring-[var(--app-primary)]`, etc.
- Disallowed in `ui/src` (except explicit, documented exceptions): `text-red-*`, `bg-white/..`, `border-black/..`, `divide-black/..`, etc.

## Component Recipes (Authoritative Patterns)

### Card
- Content cards use a single recipe (shadow vs border) consistently across the app.
- Border visibility is controlled by tokens (`--app-border`) and not by hard-coded black/white translucency.

### Muted text
- Muted/secondary text uses `--app-text-muted` semantics and must not rely on opacity.
- Provide a small helper class (e.g. `.app-text-muted`) that is safe in both light/dark and across theme presets.

### Divider / List separators
- Dividers use `--app-border` (not `divide-black/5`).
- List rows use the shared `app-list-row` recipe with token-driven hover/pressed states.

### Status / Error styling
- Status colors come from semantic tokens (`--app-success/warning/danger/info`) or from Naive UI semantic components (`NTag`, `NAlert`).
- Do not use Tailwind “red/green” color classes for meaning.

### Tables
- All `n-data-table` instances share a common class/override recipe so headers, hover, stripe, and selected states are consistent.

## Guardrails

### Scope
Guardrails apply to `ui/src/**/*.{vue,ts,tsx,css}` (excluding generated output).

### What We Check (initial set)
Fail the check if we detect:
- hard-coded Tailwind color utilities for semantic UI meaning (e.g. `text-red-*`, `bg-white/..`, `border-black/..`, `divide-black/..`),
- arbitrary font sizes (e.g. `text-[17px]`) outside documented exceptions.

Allow token-based arbitrary values referencing `var(--app-...)`.

Implementation note: keep this as a fast `rg`-based script integrated into `scripts/ci.sh` so it runs quickly and does not require additional tooling.

## Accessibility
- Muted text must remain readable in light/dark themes (avoid opacity-based contrast).
- Focus-visible ring remains standardized and token-driven (already implemented globally).
- Respect `prefers-reduced-motion` for any new animations/transitions.

