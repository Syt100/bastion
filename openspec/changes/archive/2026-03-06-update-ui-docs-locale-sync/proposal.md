# Change: Unify locale selection between Web UI and in-app docs

## Why
Today the Web UI and the in-app docs do not share a consistent language selection strategy:

- The Web UI defaults to `zh-CN` even when the user's browser prefers English.
- The docs site now supports both English and Chinese, but `/docs` is not language-aware.

For an OSS-first product, we want a predictable default (**English**) and a unified preference model so the UI and docs stay in sync.

## What Changes
- Introduce a shared locale preference cookie (e.g. `bastion_locale`) that both the Web UI and the Hub docs server understand.
- Web UI:
  - Resolve initial locale using a consistent priority order (localStorage → cookie → browser language → default `en-US`).
  - Persist locale changes to both localStorage and the shared cookie.
  - Open `/docs/` in the current locale (`/docs/` for English, `/docs/zh/` for Chinese).
- Hub docs server:
  - Make `/docs` and `/docs/` redirect to the best locale variant using `?lang=…` → cookie → `Accept-Language` → default `en-US`.
  - Set the shared locale cookie when serving HTML pages under `/docs/`.
  - Ensure redirects are not cached (and add `Vary` where appropriate) to avoid proxy cache issues.

## Impact
- Affected specs: `control-plane`, `web-ui`
- Affected code:
  - `crates/bastion-http/src/http/docs.rs`
  - `ui/src/i18n/*`, `ui/src/stores/ui.ts`, `ui/src/layouts/AppShell.vue`
  - tests for docs redirect behavior and UI locale resolution

## Non-Goals
- Server-side per-user locale settings stored in the database.
- Adding additional locales beyond `en-US` and `zh-CN`.
- Changing content translations (this change is about selection and synchronization, not new translations).

