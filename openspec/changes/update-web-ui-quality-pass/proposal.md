# Change: Update Web UI Quality Pass (Shared Styles, i18n Guardrails, A11y, Dashboard Loading)

## Why
The Web UI is functional and generally consistent, but a few quality issues reduce maintainability and polish:
- Many pages repeat the same Tailwind class strings for cards/panels, which makes visual tweaks error-prone and inconsistent over time.
- The UI supports `zh-CN` and `en-US`, but the document `<html lang>` does not track the active locale, and missing translation keys can slip in unnoticed.
- Some icon-only buttons (mobile nav/menu actions) lack accessible labels.
- The Dashboard chart area can appear blank while the chart chunk loads, which feels unresponsive on slower devices.

## What Changes
- Introduce shared UI surface styles (e.g. card/panel) as reusable CSS utilities so pages do not duplicate long class strings.
- Sync the document language attribute (`<html lang>`) with the selected UI locale.
- Add a unit test that enforces i18n key parity between `zh-CN` and `en-US` to prevent missing translations.
- Add accessible labels (`aria-label`) to icon-only buttons used in navigation/header chrome.
- Improve Dashboard chart loading UX by showing a lightweight skeleton/fallback while the chart component loads.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` styles, layout, views, i18n, and unit tests

