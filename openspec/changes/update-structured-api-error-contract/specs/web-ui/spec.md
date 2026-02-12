## ADDED Requirements

### Requirement: UI Error Resolver Must Prefer Code-Reason Localization
The Web UI SHALL resolve API error messages using structured semantics before raw backend text.

The lookup order SHALL be:
1. `apiErrors.<code>.<reason>` (with params)
2. `apiErrors.<code>`
3. backend `message`

#### Scenario: Code+reason translation overrides generic code translation
- **WHEN** an API response includes both `error` and `details.reason`
- **AND** the locale has `apiErrors.<code>.<reason>`
- **THEN** the UI displays the code+reason translation

#### Scenario: Unknown reason falls back to generic code translation
- **WHEN** an API response includes `error` and an unknown `details.reason`
- **AND** the locale has `apiErrors.<code>`
- **THEN** the UI displays the generic code translation

#### Scenario: Unknown code falls back to backend message
- **WHEN** neither `apiErrors.<code>.<reason>` nor `apiErrors.<code>` exists
- **THEN** the UI displays backend `message`

### Requirement: Form Field Errors Must Use Structured Field Mapping
Form pages SHALL use a shared mapping utility that consumes structured API error details instead of ad-hoc per-page branches.

The mapper SHALL support:
- single-field details (`field`, `reason`, `params`)
- multi-field `violations[]`

#### Scenario: Single-field validation maps deterministically
- **WHEN** API error details include `field` and `reason`
- **THEN** the corresponding form field feedback is populated through the shared mapper
- **AND** pages do not parse backend message text

#### Scenario: Multi-field violations map in one pass
- **WHEN** API error details include `violations[]`
- **THEN** the shared mapper applies errors to all referenced fields
- **AND** each field uses the same localization lookup policy

### Requirement: Path Picker Error Classification Must Not Parse Message Strings
Path picker datasource error-kind mapping SHALL use structured API codes (and reason when needed) rather than substring matching on backend message text.

#### Scenario: Filesystem/WebDAV not-directory is mapped by code
- **WHEN** picker list API returns structured not-directory error code
- **THEN** picker maps it to `not_directory` kind directly
- **AND** mapping behavior remains stable if backend message wording changes

### Requirement: Locale Packs Must Include Structured Error Keys
For migrated high-risk validation paths, locale dictionaries SHALL include reason-specific keys under `apiErrors.<code>.<reason>` in both supported languages.

#### Scenario: Setup and notification validation reasons are localizable
- **WHEN** backend returns migrated reason-specific errors for setup/auth/notification forms
- **THEN** both `en-US` and `zh-CN` provide corresponding reason-specific localization keys
