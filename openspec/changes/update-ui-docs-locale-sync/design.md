# Design: Unified locale selection (UI + docs)

## Locale identifiers

- UI locales (and cookie values): `en-US`, `zh-CN`
- Docs locales (URL paths):
  - English (default): `/docs/…`
  - Chinese: `/docs/zh/…`

Mapping:

- `zh-CN` → docs prefix `zh/`
- `en-US` → docs default (no prefix)

## Shared locale cookie

- Name: `bastion_locale`
- Value: `en-US` or `zh-CN`
- Scope: `Path=/` (so UI and docs share it)
- SameSite: `Lax`
- HttpOnly: `false` (UI JS needs to read it)
- Secure: mirror the session cookie rule (secure only when request is treated as HTTPS via trusted proxy + `X-Forwarded-Proto=https`)
- Lifetime: long-lived (e.g. 1 year) to preserve user preference

## Docs language redirect rules

Only apply language redirect on the docs root entrypoints:

- `GET /docs`
- `GET /docs/`

Priority order:

1. `?lang=…` (explicit override; accepted: `en`, `en-US`, `zh`, `zh-CN`)
2. `Cookie: bastion_locale=…`
3. `Accept-Language` (treat any `zh*` as Chinese; otherwise English)
4. Default English

Redirect targets:

- English: `/docs/`
- Chinese: `/docs/zh/`

Cache safety:

- Redirect responses MUST include `Cache-Control: no-store`
- Add `Vary: Accept-Language, Cookie` to prevent incorrect cache sharing across users

Cookie writeback:

- When serving an HTML docs page, set `bastion_locale` based on the requested docs locale path.
  - `/docs/zh/*` → `zh-CN`
  - otherwise → `en-US`

## Web UI locale resolution

Single shared resolution logic (used for both `vue-i18n` and the UI store):

1. localStorage `bastion.ui.locale` (if supported)
2. cookie `bastion_locale` (if supported)
3. browser language (`navigator.languages` / `navigator.language`)
   - any `zh*` → `zh-CN`, else `en-US`
4. default `en-US`

Persistence:

- On first resolution when no stored preference exists, write both localStorage + cookie.
- When the user changes language in the UI, write both localStorage + cookie.

Docs entry:

- The UI "Help" action opens the locale-specific docs root:
  - `zh-CN` → `/docs/zh/`
  - `en-US` → `/docs/`

