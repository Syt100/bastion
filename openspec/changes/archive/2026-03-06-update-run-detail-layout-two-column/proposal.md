# Change: Run Detail Desktop Two-Column Layout + Productized Header/Actions

## Why
The Run Detail page is functional but still feels "flat" on desktop: key information competes vertically with details, and the primary actions/status/target presentation is not cohesive.

## What Changes
- Desktop uses a two-column layout:
  - Left: a compact "Summary + Progress" panel (default expanded, stays visible while browsing details).
  - Right: Details tabs (Events / Operations / Summary).
- The header/action area is presented as a cohesive product UI:
  - Run status uses localized labels.
  - Target type is shown with human-friendly labeling.
  - Primary actions (Refresh / Restore / Verify / More) are grouped and have consistent disabled states.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/views/RunDetailView.vue`, run detail components, i18n strings
- No backend changes
