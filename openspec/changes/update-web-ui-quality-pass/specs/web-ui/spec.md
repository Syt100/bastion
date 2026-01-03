## ADDED Requirements

### Requirement: Shared UI Surface Styles
The Web UI SHALL centralize common surface styles (e.g. card/panel appearance) so pages and components can reuse them without duplicating long class strings.

#### Scenario: Card surface style is updated in one place
- **WHEN** the card/panel surface appearance needs to change (border/shadow/contrast)
- **THEN** the change can be made in a single shared style utility
- **AND** pages using that utility automatically inherit the updated appearance

### Requirement: Document Language Tracks UI Locale
The Web UI SHALL keep the document `<html lang>` attribute synchronized with the active UI locale.

#### Scenario: Switching locale updates document lang
- **WHEN** the user changes the UI language from `zh-CN` to `en-US`
- **THEN** the document `<html lang>` attribute becomes `en`

### Requirement: i18n Key Parity Is Enforced
The Web UI SHALL include an automated check that enforces i18n key parity between supported locales (`zh-CN` and `en-US`) to prevent missing translation keys.

#### Scenario: Missing translation key fails tests
- **WHEN** a translation key exists in `zh-CN` but not in `en-US` (or vice-versa)
- **THEN** the UI unit tests fail and report the missing key(s)

### Requirement: Icon-Only Buttons Are Accessible
Icon-only buttons in the global navigation/header chrome SHALL include accessible labels so they can be understood by assistive technology.

#### Scenario: Mobile hamburger button has an accessible label
- **WHEN** the mobile navigation hamburger button is rendered
- **THEN** it includes an `aria-label` describing its action (localized)

### Requirement: Dashboard Chart Shows Loading Fallback
The Dashboard chart area SHALL display a lightweight fallback UI while the async chart component is loading.

#### Scenario: Chart does not render as a blank area while loading
- **WHEN** the Dashboard page first renders and the chart chunk has not loaded yet
- **THEN** a visible fallback (e.g. skeleton/placeholder) is shown until the chart is ready

