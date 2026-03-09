## MODIFIED Requirements

### Requirement: UI Provides A Help Entry Point To Product Docs (Locale-Aware)
The Web UI SHALL provide a visible "Help" entry point that opens the product documentation under `/docs/` in the user's selected locale.

#### Scenario: User opens docs in Chinese from the UI
- **GIVEN** the UI locale is `zh-CN`
- **WHEN** the user clicks "Help"
- **THEN** the browser opens `/docs/zh/`

#### Scenario: User opens docs in English from the UI
- **GIVEN** the UI locale is `en-US`
- **WHEN** the user clicks "Help"
- **THEN** the browser opens `/docs/`

## ADDED Requirements

### Requirement: UI Resolves Initial Locale Using A Unified Strategy
On first load, the Web UI SHALL resolve the initial locale using the following priority order:

1. localStorage `bastion.ui.locale`
2. cookie `bastion_locale`
3. browser language (`navigator.languages` / `navigator.language`)
4. default English (`en-US`)

#### Scenario: Stored locale in localStorage wins
- **GIVEN** localStorage contains `bastion.ui.locale=en-US`
- **WHEN** the UI loads
- **THEN** the UI uses locale `en-US`

#### Scenario: Cookie is used when localStorage is missing
- **GIVEN** localStorage does not contain `bastion.ui.locale`
- **AND GIVEN** cookie contains `bastion_locale=zh-CN`
- **WHEN** the UI loads
- **THEN** the UI uses locale `zh-CN`

