# Change: Add Web UI Theme Presets (Fresh Mint Teal Default + 5 Alternatives)

## Why
The current Web UI palette is usable but does not feel "fresh / bright" enough, especially in dark mode:
- Light mode lacks a clean, lively accent system that feels modern and approachable.
- Dark mode reads "muddy" due to low contrast and gray-ish surfaces, reducing perceived quality.

We want a lightweight theme system that lets users choose from a small set of curated color schemes (no custom colors yet), with a new default theme aligned to a "mint + teal" fresh look.

## What Changes
- Add a Web UI theme preset system (6 curated themes) that switches shared design tokens (light + dark).
- Make **Mint Teal** the default theme: fresh mint background in light mode, and higher-contrast "blacker black / whiter white" in dark mode.
- Allow each theme to define its own background "aurora" gradient to reinforce personality without reducing readability.
- Provide a theme picker UI in Settings (mobile-friendly), showing theme preview swatches and names.
- Persist the selected theme across sessions; fall back to the default if an unknown value is stored.
- Ensure Naive UI theme overrides update correctly on theme change and remain robust (no `var(...)` strings passed into overrides).
- Update the browser `theme-color` meta to reflect the active theme (light uses accent; dark uses background).

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/styles` (tokens), `ui/src/stores/ui.ts` (preference), `ui/src/App.vue` (theme application), Settings UI (theme picker), and UI tests

