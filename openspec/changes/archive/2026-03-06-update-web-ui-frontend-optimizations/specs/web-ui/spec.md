## ADDED Requirements

### Requirement: Route-Level Code Splitting
The Web UI SHALL lazy-load page views using dynamic imports to reduce the initial JS payload.

#### Scenario: Views are lazy-loaded
- **WHEN** routes are defined
- **THEN** view components are referenced via dynamic imports rather than eager imports

### Requirement: Deferred Chart Loading
The Web UI SHALL defer loading charting libraries until the Dashboard requires them.

#### Scenario: Chart library is not loaded on login
- **WHEN** the user is on the Login page
- **THEN** the chart implementation is not loaded

### Requirement: Jobs Page Modularity
The Jobs page implementation SHALL be decomposed into focused components/composables to keep complexity manageable.

#### Scenario: Jobs editor is implemented as a dedicated component
- **WHEN** a user creates or edits a job
- **THEN** the editor modal/wizard is implemented in a dedicated component (not a single monolithic view file)

### Requirement: Shared Utilities
The Web UI SHALL centralize common formatting and utility logic to avoid duplication.

#### Scenario: Unix timestamps are formatted consistently
- **WHEN** Unix timestamps are rendered across pages
- **THEN** they use a shared formatter helper to ensure consistent locale-aware output

### Requirement: Centralized CSRF Handling
State-changing API requests in the Web UI SHALL include `X-CSRF-Token` via a shared helper.

#### Scenario: Create job attaches CSRF token
- **WHEN** a job is created via the UI
- **THEN** the request includes the `X-CSRF-Token` header

### Requirement: Session Expiry Handling
When the backend returns `401 Unauthorized` to a UI request, the Web UI SHALL transition to anonymous state and redirect the user to the Login page.

#### Scenario: 401 triggers login redirect
- **WHEN** an authenticated user receives `401 Unauthorized` from an API call
- **THEN** the UI redirects to `/login`

### Requirement: Theme Defaults Follow System
If no explicit theme preference is stored, the Web UI SHALL default to the system theme preference (`prefers-color-scheme`).

#### Scenario: System prefers dark with no stored preference
- **WHEN** no theme preference exists in local storage and the system preference is dark
- **THEN** the UI initializes in dark mode

### Requirement: Chart Labels Are Localized
Any demo chart labels shown in the Dashboard SHALL be localized for `zh-CN` and `en-US`.

#### Scenario: Chart legend uses localized labels
- **WHEN** the Dashboard chart is rendered in different locales
- **THEN** legend/axis labels reflect the selected locale
