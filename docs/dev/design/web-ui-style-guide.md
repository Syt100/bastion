# Web UI style guide (design system)

This guide documents the *authoritative* visual rules for the Bastion Web UI.
It is written for contributors: if you are adding or modifying UI, follow these rules to keep the product consistent across pages, light/dark mode, and theme presets.

## Goals

- Keep the UI visually consistent across features, pages, and contributors.
- Ensure light/dark mode and theme presets stay correct by default.
- Make the consistent choice the easiest choice: tokens first, then shared recipes, then local tweaks.

## Where the truth lives

- Theme + tokens: `ui/src/styles/main.css`
- Naive UI theme integration: `ui/src/App.vue`
- Shared layout patterns: `ui/src/layouts/AppShell.vue`, `ui/src/components/*`

If you need a new color/shadow/surface/etc, add a token (or reuse an existing one). Do not hard-code colors in views.

## Token model

### Theme tokens (per preset)

Theme presets define the core palette (light + dark) via CSS variables on the document root:

- Light: `[data-theme="..."] { ... }`
- Dark: `.dark[data-theme="..."] { ... }`

These tokens include (non-exhaustive):

- Accents: `--app-primary`, `--app-primary-soft`, `--app-primary-2`
- Text: `--app-text`, `--app-text-muted`
- Surfaces: `--app-bg-solid`, `--app-surface`, `--app-surface-2`, `--app-bg` (aurora layers)
- Borders + states: `--app-border`, `--app-hover`, `--app-pressed`
- Status: `--app-info`, `--app-success`, `--app-warning`, `--app-danger` (global semantics)

### Foundation tokens (global)

Foundation tokens are *not* theme-specific. They define stable primitives that should not change per theme:

- Radii: `--app-radius-sm`, `--app-radius`, `--app-radius-lg`
- Motion: `--app-duration-fast`, `--app-duration-normal`, `--app-ease-standard`

## Tailwind usage rules (long-term)

### Allowed

- Layout/spacing/typography via Tailwind scales (avoid arbitrary values):
  - `p-4`, `gap-3`, `text-sm`, `rounded-xl`, etc.
- Token-driven colors via `var(--app-...)`:
  - `bg-[var(--app-surface-2)]`
  - `text-[var(--app-danger)]`
  - `border-[color:var(--app-border)]`

### Avoid / disallowed (unless explicitly documented)

- Hard-coded palette colors: `text-red-*`, `bg-amber-*`, `border-slate-*`, `dark:text-blue-*`, etc.
- Opacity-based muted semantics for text: `opacity-70` / `opacity-80` (use `--app-text-muted` instead).
- Theme-incompatible chrome colors: `bg-white/..`, `border-black/..`, `divide-black/..`, etc.

## Semantic utility classes

Prefer these shared classes instead of re-creating ad-hoc patterns:

- `app-text-muted`: secondary/muted text color.
- `app-border-subtle`: subtle 1px border using `--app-border`.
- `app-divide-y`: token-driven list separators between direct children.
- `app-panel-inset`: inset surface (`--app-surface-2`) + subtle border.
- `app-card`: shared card elevation.
- `app-list-row`: shared clickable row pattern (spacing + hover).
- `app-mono-block`: mono panel for IDs / paths / snippets.
- `app-kbd`: keyboard keycap styling for shortcuts.
- `app-glass`, `app-glass-soft`: glass surfaces for navigation chrome (use sparingly).

## Component recipes

### Card (content container)

Default content card:

```vue
<n-card class="app-card" :bordered="false">
  ...
</n-card>
```

Rule: if you use `class="app-card"` on an `n-card`, always set `:bordered="false"`. The elevation is provided by `app-card`, and mixing borders in some places but not others is one of the easiest ways for the UI to drift over time.

Inset panel inside a card:

```vue
<div class="rounded app-panel-inset p-3">
  ...
</div>
```

### Muted text

Use token-driven muted color:

```html
<div class="text-sm app-text-muted">Help text…</div>
```

Do not use opacity to “fake” muted text:

```html
<!-- don't -->
<div class="text-sm opacity-70">Help text…</div>
```

### List separators

Use `app-divide-y` on the list container:

```html
<div class="app-divide-y">
  <button class="app-list-row">...</button>
  <button class="app-list-row">...</button>
</div>
```

### Status and errors

Prefer semantic Naive UI components:

```vue
<n-alert type="error" :bordered="false">...</n-alert>
<n-tag size="small" :bordered="false" type="warning">...</n-tag>
```

If you need plain text coloring, use tokens:

```html
<div class="text-xs text-[var(--app-danger)]">Error message…</div>
```

### Tables

All data tables should share the same visual rules (header, hover, selected state). Prefer applying a shared class consistently (see `ui/src/styles/main.css`).

### Monospace blocks and keycaps

```html
<div class="app-mono-block break-all">{{ id }}</div>
<kbd class="app-kbd">Ctrl</kbd>
```

## Review checklist

- Uses `--app-*` tokens for semantic colors (not Tailwind palette colors).
- Uses `app-text-muted` for secondary text (not opacity).
- Uses `app-divide-y` / `app-border-subtle` for dividers and subtle borders.
- Uses the shared recipes (`app-card`, `app-list-row`, toolbars) instead of re-inventing patterns.
- Works in light + dark mode and across theme presets.
