## Context
This change introduces a small, curated theme system for the Web UI. Themes are **preset** color schemes only (no custom theme editor). The selected theme must work across desktop + mobile and remain consistent with existing design tokens and Naive UI theming.

## Goals / Non-Goals

### aligned Goals
- Provide 6 preset themes and allow users to switch between them instantly.
- Make the default palette "fresh, bright" with a mint base and teal primary accents.
- Improve dark mode perceived quality: darker backgrounds, whiter text, clearer surface hierarchy, and reduced "muddy" grays.
- Allow themes to customize the background aurora/gradient for personality.
- Keep the implementation simple: CSS variables + a small preference in the UI store.

### Non-Goals
- No user-defined custom colors, no theme editor, no importing/exporting themes.
- No server-side theme preference synchronization in this iteration.
- No change to functional workflows beyond adding theme selection and updating colors.

## Theme Model

### Identifiers
Themes are identified by a stable `themeId` string, stored client-side.

Initial theme set (6):
- `mint-teal` (default): Mint base + teal primary + cyan secondary, lively + clean.
- `ocean-blue`: Ice/sky base + blue primary + cyan secondary, crisp + modern.
- `grape-violet`: Light lavender base + indigo/violet primary, elegant but still bright.
- `sunset-amber`: Warm cream base + amber/orange primary, energetic and friendly.
- `berry-rose`: Soft blush base + rose primary, vivid but controlled.
- `coral-peach`: Peach base + coral primary, warm and playful.

### Where Theme Is Applied
Apply the theme to the document root using a data attribute:
- `<html data-theme="mint-teal">`
- Dark mode continues to use the existing `.dark` class on `<html>` (and `<body>` for compatibility).

This yields selectors like:
- `:root[data-theme="mint-teal"] { ... }` (light)
- `.dark[data-theme="mint-teal"] { ... }` (dark)

## Token Strategy

### Tokens that themes MUST define
Each theme provides both light and dark values for:
- Neutral/surfaces: `--app-bg-solid`, `--app-surface`, `--app-surface-2`
- Text: `--app-text`, `--app-text-muted`
- Borders + states: `--app-border`, `--app-hover`, `--app-pressed`
- Accents: `--app-primary`, `--app-primary-hover`, `--app-primary-pressed`, `--app-primary-soft`
- Secondary accent: `--app-primary-2`, `--app-primary-2-soft`
- Background image stack: `--app-bg` (aurora gradient images only; solid base comes from `--app-bg-solid`)
- Focus ring: `--app-focus`

### Tokens that SHOULD remain global (not theme-specific)
Status semantics SHOULD remain consistent across themes to avoid confusion:
- `--app-success`, `--app-warning`, `--app-danger`, `--app-info`

## Accessibility and Contrast Targets
- Dark mode MUST increase perceived contrast compared to the current palette:
  - Backgrounds closer to near-black (reduced gray haze).
  - Primary text closer to near-white for readability.
- Normal body text SHOULD meet WCAG AA contrast against its surface.
- Background aurora gradients MUST stay subtle and MUST NOT reduce legibility of text/content.

## Naive UI Integration Notes
The app derives Naive UI `themeOverrides` from computed CSS values. Theme switching must:
- Recompute overrides when `themeId` changes.
- Continue to avoid passing raw `var(...)` strings into overrides (seemly parsing constraint).

## Settings UI
Add a new "Appearance" section under Settings:
- Shows theme cards with name + swatch preview (accent + background).
- Mobile: single-column list; Desktop: 2–3 columns.
- Selecting a theme applies it immediately and persists preference.

