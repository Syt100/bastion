## MODIFIED Requirements

### Requirement: Hub Serves Product Documentation Under /docs (Locale-Aware)
The Hub SHALL serve a static product documentation site under `/docs/` without requiring authentication by default.

The Hub SHALL make the docs entrypoint locale-aware:

- `/docs` and `/docs/` MUST resolve a preferred docs locale.
- The default locale MUST be English.

Preferred locale resolution order:

1. `?lang=…` (explicit; `en|en-US|zh|zh-CN`)
2. `Cookie: bastion_locale=…` (`en-US|zh-CN`)
3. `Accept-Language` (any `zh*` chooses Chinese; otherwise English)
4. Default English

#### Scenario: Docs entrypoint redirects to Chinese when requested
- **WHEN** a client requests `GET /docs/` with `Accept-Language: zh-CN`
- **THEN** the Hub redirects to `/docs/zh/`

#### Scenario: Docs entrypoint stays English by default
- **WHEN** a client requests `GET /docs/` with no `Accept-Language` and no locale cookie
- **THEN** the Hub serves the English docs index (no redirect to `/docs/zh/`)

#### Scenario: Explicit query param overrides cookie and Accept-Language
- **GIVEN** the client has `Cookie: bastion_locale=zh-CN`
- **WHEN** a client requests `GET /docs/?lang=en`
- **THEN** the Hub redirects to `/docs/`

### Requirement: Docs Locale Redirects Are Cache-Safe
Redirect responses for docs locale selection MUST NOT be cached in a way that can affect other users.

#### Scenario: Proxy-safe locale redirect
- **WHEN** a client requests `GET /docs/` and receives a locale redirect
- **THEN** the response includes `Cache-Control: no-store`
- **AND** the response includes `Vary: Accept-Language, Cookie`

## ADDED Requirements

### Requirement: Hub Persists Locale Preference For Docs
When serving an HTML docs page, the Hub SHALL set a shared locale preference cookie so the Web UI and docs can remain in sync.

- Cookie name: `bastion_locale`
- Cookie values: `en-US|zh-CN`

#### Scenario: Visiting the Chinese docs sets locale cookie
- **WHEN** a client requests an HTML page under `/docs/zh/`
- **THEN** the response sets `bastion_locale=zh-CN`

