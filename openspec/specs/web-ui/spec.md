# web-ui Specification

## Purpose
TBD - created by archiving change update-jobs-filters-and-row-actions-discoverability. Update Purpose after archive.
## Requirements
### Requirement: Jobs Filters Use A Single Source Of Truth
The Jobs workspace SHALL use one canonical filter state model for search, archived toggle, latest-run status, schedule mode, and sort order across all layout modes.

#### Scenario: Split/list/mobile layouts share the same filter values
- **GIVEN** the user has set Jobs filters in one layout mode
- **WHEN** the layout mode changes (split, full list, or mobile list)
- **THEN** the same filter values remain active
- **AND** the fetched results reflect the same criteria without mode-specific divergence

### Requirement: Jobs Filter Controls Render Consistently Across Containers
The Jobs workspace SHALL reuse the same filter control definitions across inline, popover, and drawer containers.

#### Scenario: Filter controls keep parity across container types
- **GIVEN** Jobs filters are shown in inline (desktop), popover (split), or drawer (mobile) container
- **WHEN** the user opens the filter controls
- **THEN** the available controls, option sets, and clear behavior are equivalent
- **AND** active-filter count/chips are derived from the same filter model

### Requirement: Primary Job Row Actions Are Discoverable Without Hover
The Jobs list SHALL expose primary row actions in a persistently visible way, without requiring hover-only reveal.

#### Scenario: User can trigger primary row actions immediately
- **GIVEN** the user views the Jobs list on pointer, touch, or keyboard workflow
- **WHEN** a row is visible
- **THEN** at least the primary action set (for example, Run Now and More) is directly visible
- **AND** the user does not need hover to discover these primary actions

### Requirement: Row Action Interaction Boundaries Are Explicit
Jobs row navigation and row action buttons SHALL have non-conflicting interaction boundaries.

#### Scenario: Action click does not trigger row navigation
- **GIVEN** a user activates a job row action button
- **WHEN** the action is triggered
- **THEN** only that action executes
- **AND** the row does not simultaneously open job detail navigation

#### Scenario: Row click still opens job detail
- **GIVEN** a user clicks the non-action area of a job row
- **WHEN** row click behavior runs
- **THEN** the Jobs workspace opens the selected job detail as before

### Requirement: List Pages Use A Shared Structural Scaffold
The Web UI SHALL provide a shared list-page scaffold that standardizes the structure of selection tools, filter/action toolbar, list content region, and pagination footer across list-oriented pages.

#### Scenario: Shared scaffold regions are rendered consistently
- **GIVEN** a list-oriented page that opts into the shared scaffold
- **WHEN** the page renders in desktop or mobile layout
- **THEN** selection tools (if any), toolbar controls, content area, and pagination footer are rendered in stable, predictable regions
- **AND** page-specific business widgets are injected through explicit slots/regions instead of custom page wrappers

### Requirement: List Pagination Interaction Is Consistent Across Pages
List-oriented pages SHALL expose consistent pagination behavior and controls, including page number, page-size selection, total count visibility, and disabled/loading states.

#### Scenario: Notifications queue follows the same pagination pattern as Jobs and Agents
- **GIVEN** the user is viewing Notifications Queue, Jobs list, or Agents list
- **WHEN** the user navigates pages or changes page size
- **THEN** each page presents equivalent pagination controls and semantics
- **AND** pagination disabled/loading behavior is consistent across these pages

#### Scenario: Filter changes reset pagination predictably
- **GIVEN** a user is on page N of a list
- **WHEN** the user updates search/filter criteria
- **THEN** the list resets to the first page before fetching updated results
- **AND** the behavior is consistent for all migrated list pages

### Requirement: Empty States Support Context-Aware Surface Variants
The Web UI SHALL provide empty-state rendering variants that avoid nested card surfaces when an empty state is rendered inside an existing card/list container.

#### Scenario: Empty state inside list card does not create nested card noise
- **GIVEN** a list page content area already uses a card/surface container
- **WHEN** the list enters loading-empty or no-data state
- **THEN** the empty state uses a non-card variant (`plain` or `inset`)
- **AND** the page does not render a card-within-card empty-state pattern

### Requirement: List Visual Hierarchy Avoids Redundant Decorative Layers
List pages SHALL avoid stacking redundant decorative wrappers around toolbars and empty/content regions, preserving a single dominant surface hierarchy per section.

#### Scenario: Toolbar and empty/list content do not create stacked decorative wrappers
- **GIVEN** a migrated list page with a primary card/surface
- **WHEN** toolbars and empty states are rendered
- **THEN** the page uses at most one dominant elevated surface for the list section
- **AND** additional wrappers are neutral/inset rather than repeated elevated cards

### Requirement: Run-event detail dialogs SHALL prioritize concise diagnostics before verbose payloads
The Web UI SHALL present actionable diagnostics (localized message/hint, key envelope rows, and concise context) before rendering verbose raw payload sections.

#### Scenario: Event detail opens with envelope and raw fields
- **GIVEN** a run event contains `error_envelope` and additional raw `fields`
- **WHEN** a user opens event details from either run-detail page or run-events modal
- **THEN** the dialog SHALL show concise diagnostics first
- **AND** verbose raw JSON SHALL remain collapsed until the user explicitly expands it

### Requirement: Run-event detail dialogs SHALL support progressive disclosure for long error chains
The Web UI SHALL provide an expandable view for long `error_chain` diagnostics to reduce first-screen noise while keeping full detail available on demand.

#### Scenario: Long error chain exists
- **GIVEN** event fields include an `error_chain` list with multiple entries
- **WHEN** the detail dialog is rendered
- **THEN** the dialog SHALL show a compact preview first
- **AND** provide explicit expand/collapse actions to view full chain entries

### Requirement: Shared event-detail renderer SHALL keep behavior consistent across entry points
Run detail page and run-events modal SHALL reuse the same event-detail content behavior for diagnostics ordering, expansion controls, and raw payload rendering.

#### Scenario: User opens the same event from two entry points
- **GIVEN** an event is reachable from both run-detail and run-events modal
- **WHEN** user opens event details from each entry point
- **THEN** both dialogs SHALL render the same content sections and progressive disclosure controls

### Requirement: Web UI SHALL render canonical error envelope diagnostics
Web UI diagnostics panels SHALL prioritize canonical envelope fields when present.

#### Scenario: Event contains canonical envelope
- **GIVEN** a run event includes canonical envelope diagnostics
- **WHEN** user opens event details
- **THEN** UI SHALL render localized message and hint from envelope keys and params
- **AND** UI SHALL fall back gracefully when localization keys are missing

### Requirement: Web UI SHALL display protocol-specific details by transport
Web UI SHALL render transport-specific detail rows using envelope transport metadata.

#### Scenario: HTTP event shows HTTP diagnostics
- **GIVEN** envelope `transport.protocol` is `http`
- **WHEN** event details are rendered
- **THEN** UI SHALL display HTTP-specific diagnostics (for example status and retry delay) when available

#### Scenario: SFTP event shows provider diagnostics without HTTP fields
- **GIVEN** envelope `transport.protocol` is `sftp`
- **WHEN** event details are rendered
- **THEN** UI SHALL display SFTP/provider diagnostic fields
- **AND** UI SHALL NOT require HTTP-specific fields to render meaningful diagnostics

### Requirement: Web UI SHALL expose async-operation and partial-failure context
Web UI SHALL expose operation-level and partial-failure diagnostics when provided by the envelope.

#### Scenario: Async operation context is present
- **GIVEN** envelope context includes async operation metadata
- **WHEN** user inspects details
- **THEN** UI SHALL display operation id, current status, and next polling hint when available

#### Scenario: Partial failures are present
- **GIVEN** envelope context includes partial failure items
- **WHEN** user inspects details
- **THEN** UI SHALL render a per-item diagnostic list with resource id/path and error summary

### Requirement: Run event hint labels are localized
The Web UI SHALL localize run-event detail hint labels for all supported locales.

#### Scenario: Chinese locale renders localized hint label
- **GIVEN** UI locale is `zh-CN`
- **AND** run event detail contains a `hint` field
- **WHEN** user opens event details
- **THEN** the hint label SHALL be displayed in Chinese instead of hardcoded English text

### Requirement: Hint rendering is source-agnostic
The Web UI SHALL render hint text whenever `hint` is present, regardless of whether it originates from run failures or cleanup maintenance events.

#### Scenario: Cleanup event provides hint field
- **GIVEN** a run event from cleanup/maintenance pipeline includes `fields.hint`
- **WHEN** user opens run event details
- **THEN** UI SHALL render the hint block with localized label and original hint content

### Requirement: Run events UI highlights actionable failure diagnostics
The Web UI SHALL prominently present actionable diagnostics from failed run events without requiring users to inspect raw JSON.

#### Scenario: Failed event includes operator hint
- **GIVEN** a run failed event contains `hint` and classification fields
- **WHEN** user opens run events
- **THEN** UI displays hint/diagnostic cues in list chips or equivalent compact markers
- **AND** raw JSON details remain available in the event detail panel

### Requirement: Run events UI surfaces transport metadata for upload failures
The Web UI SHALL surface key transport metadata for upload-related failures when available.

#### Scenario: Upload failure includes HTTP and part metadata
- **GIVEN** failed event fields include HTTP status and part size/name
- **WHEN** user inspects the run events list/details
- **THEN** UI displays recognizable status/part diagnostics to speed troubleshooting

### Requirement: Web UI Exposes Cancel Actions For Runs and Operations
The Web UI SHALL provide cancel actions for eligible run/operation states and call backend cancel APIs.

#### Scenario: Cancel button is available for queued/running run
- **GIVEN** a run is `queued` or `running`
- **WHEN** the operator opens run details or list actions
- **THEN** the UI shows a cancel action and triggers run cancel API on confirmation

#### Scenario: Cancel button is available for running restore/verify operation
- **GIVEN** an operation is `running`
- **WHEN** the operator opens operation details
- **THEN** the UI shows a cancel action and triggers operation cancel API on confirmation

### Requirement: Web UI Shows Cancel-In-Progress and Canceled Terminal States
The Web UI SHALL represent cancellation lifecycle clearly for both runs and operations.

#### Scenario: Cancel requested while task still running
- **GIVEN** cancel has been requested for a running run/operation
- **WHEN** terminal `canceled` has not yet been reached
- **THEN** the UI displays a cancel-in-progress state and disables duplicate action spam

#### Scenario: Task reaches terminal canceled
- **WHEN** run/operation status becomes `canceled`
- **THEN** the UI renders terminal canceled badge/status and hides actions that require active execution

### Requirement: Cancel Mutation Handling Is Idempotent In UI State Stores
Web UI stores SHALL handle repeated cancel clicks and repeated cancel responses without inconsistent local state.

#### Scenario: User clicks cancel multiple times
- **WHEN** the user clicks cancel repeatedly before the first response returns
- **THEN** only one effective cancel mutation is processed and local status remains consistent

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

### Requirement: Dashboard Defers Heavy Desktop Recent-Runs Table Module
The dashboard SHALL defer loading the desktop recent-runs table module until the recent-runs section is near or inside the viewport.

#### Scenario: Desktop recent table module loads on visibility
- **GIVEN** the dashboard has recent run records
- **AND** viewport is desktop-sized
- **WHEN** the recent section has not yet entered viewport proximity
- **THEN** the heavy desktop table module is not rendered yet
- **AND** the dashboard shows a lightweight deferred-loading placeholder for that section
- **WHEN** the section enters viewport proximity
- **THEN** the desktop table module is loaded and rendered

### Requirement: Dashboard Uses Shared Viewport-Lazy Activation Helper
Dashboard viewport-lazy section activation logic SHALL use a shared helper to keep behavior consistent and reduce duplicated observer lifecycle code.

#### Scenario: Shared helper activates section once visible and handles cleanup
- **GIVEN** a section is configured with viewport-lazy activation
- **WHEN** the target element intersects the viewport
- **THEN** the section is marked ready and observer is disconnected
- **AND** helper cleanup disconnects observers when no longer needed

### Requirement: Deferred Dashboard Sections Show Lightweight Animated Loading Feedback
When deferred dashboard sections are waiting for module activation, the UI SHALL display lightweight animated loading feedback in addition to skeleton placeholders.

#### Scenario: Deferred section placeholder includes animated loading indicator
- **GIVEN** a deferred dashboard section is not yet ready to render
- **WHEN** the placeholder is shown
- **THEN** users can see an animated loading indicator
- **AND** no heavy animation assets are required to render the indicator

### Requirement: Agents View Uses Remote Pagination and Remote Core Filters
Agents view SHALL request paginated agent data from backend using current page/page_size and current filter state (labels, labels mode, status, search query) instead of client-side full-list filtering.

#### Scenario: Changing page triggers remote fetch
- **GIVEN** the agents list is displayed
- **WHEN** the user changes page or page size
- **THEN** the view requests `/api/agents` with updated pagination query params
- **AND** pagination UI uses server `total` to render controls

#### Scenario: Changing status or search triggers remote fetch
- **GIVEN** agents list filters are visible
- **WHEN** the user updates status/search filters
- **THEN** the view resets to page 1 and requests server-filtered data
- **AND** list rendering uses returned `items` directly

### Requirement: Jobs Workspace List Uses Remote Filtering/Sorting/Pagination
Jobs workspace list SHALL execute filtering, sorting, and pagination through `/api/jobs` query params based on current node context and filter controls.

#### Scenario: List filters update backend query
- **GIVEN** the jobs workspace list is open
- **WHEN** the user changes archived toggle/search/latest-status/schedule/sort or node context
- **THEN** the view refreshes from backend with matching query params
- **AND** list/table render only returned page items

#### Scenario: Pagination controls reflect backend total
- **GIVEN** the backend reports paged list metadata
- **WHEN** the jobs workspace renders pagination
- **THEN** `item-count` is sourced from backend `total`
- **AND** page changes request the corresponding server page

### Requirement: Dashboard Provides First-Paint Skeleton and Viewport-Lazy Chart Rendering
The dashboard SHALL provide immediate first-paint skeleton loading placeholders for key summary sections and SHALL defer rendering the trend chart component until the chart container is near or inside the viewport.

#### Scenario: First dashboard paint shows skeleton placeholders
- **GIVEN** a user opens the dashboard and overview data is still loading
- **WHEN** the dashboard first renders
- **THEN** summary sections render skeleton placeholders instead of empty or misleading zero values
- **AND** loading UI remains responsive until overview data is available

#### Scenario: Trend chart render is deferred by viewport visibility
- **GIVEN** dashboard trend data exists
- **WHEN** the chart section has not yet entered the viewport
- **THEN** heavy chart rendering is deferred
- **AND** once the chart section enters (or nears) the viewport, the chart renders with loading fallback

### Requirement: Stores Use Shared Latest-Request Cancellation Semantics
Web UI stores that refresh list/overview data SHALL use a shared latest-request cancellation mechanism so that superseded in-flight requests are canceled and stale responses cannot overwrite newer state.

#### Scenario: Overlapping dashboard refresh only keeps latest response
- **GIVEN** dashboard refresh request A is in flight
- **AND** request B starts before A completes
- **WHEN** request B completes successfully
- **THEN** request A is canceled or treated stale
- **AND** dashboard state reflects request B only

### Requirement: Jobs and Agents Views Support Paged Rendering
Jobs and agents list presentations SHALL support client-side pagination to limit rows/cards rendered at once while preserving filter/sort semantics.

#### Scenario: Large filtered result set renders current page only
- **GIVEN** a filtered list returns many rows
- **WHEN** the user views jobs or agents list
- **THEN** only items for the selected page are rendered
- **AND** pagination controls allow changing page and page size

### Requirement: Dashboard Defers Heavy Chart Module Loading
The dashboard view SHALL defer loading heavy visualization modules until chart data needs to be rendered, and SHALL show a loading fallback while the chart module resolves.

#### Scenario: Dashboard trend chart is lazy-loaded
- **GIVEN** a user navigates to the dashboard
- **AND** trend data exists for the 7-day chart
- **WHEN** the dashboard view renders
- **THEN** the chart visualization component is loaded asynchronously
- **AND** the dashboard shows a loading fallback until the chart module is ready

### Requirement: List Refresh Cancels Superseded Requests
For list refresh workflows in Web UI state stores, the system SHALL cancel superseded in-flight refresh requests when a newer refresh starts.

#### Scenario: New refresh cancels previous in-flight request
- **GIVEN** a list view triggers refresh request A
- **AND** request A is still in flight
- **WHEN** the user triggers refresh request B
- **THEN** request A is canceled
- **AND** only request B may update the active list state

### Requirement: List Refresh Uses Latest-Request-Wins Semantics
For list refresh workflows in Web UI state stores, the system SHALL apply latest-request-wins semantics:
- when multiple refresh requests overlap, only the most recent request may update list data and loading state
- stale request failures SHALL be ignored and MUST NOT overwrite newer successful state

#### Scenario: Stale success response cannot overwrite newer result
- **GIVEN** a list view triggers refresh request A
- **AND** before A returns, the user triggers refresh request B
- **AND** request B completes successfully first with dataset B
- **WHEN** request A completes later with dataset A
- **THEN** the store keeps dataset B
- **AND** the stale dataset A is ignored

#### Scenario: Stale failure cannot override newer success
- **GIVEN** a list view triggers refresh request A
- **AND** before A returns, the user triggers refresh request B
- **AND** request B completes successfully first
- **WHEN** request A fails later
- **THEN** the store keeps the successful state from request B
- **AND** the stale failure from request A does not become the active list state

### Requirement: Jobs Workspace Provides Clear Refresh Controls
The Web UI SHALL provide distinct, accessible refresh actions for:
- refreshing the jobs list, and
- refreshing the selected job's detail.

#### Scenario: User can tell list refresh apart from detail refresh
- **GIVEN** the user is on a desktop-sized screen
- **AND** the user is in the Jobs workspace in split layout
- **AND** a job is selected (detail is visible)
- **WHEN** the user views the available refresh actions
- **THEN** the UI provides clear labels and/or tooltips that distinguish refreshing the list from refreshing the selected job
- **AND** both actions provide accessible labels (e.g. `aria-label`)

### Requirement: Jobs Workspace Shows Active Filters and Result Counts
The Web UI SHALL show a compact summary of the jobs list filtering state, including:
- a results counter (filtered count and total count), and
- active filter chips that can be removed individually, and
- a clear-all filters action.

#### Scenario: User removes one active filter chip
- **GIVEN** the user has applied at least one jobs list filter
- **WHEN** the user closes an active filter chip
- **THEN** only that corresponding filter is cleared
- **AND** the jobs list results update accordingly

### Requirement: Jobs Workspace Supports Bulk Selection and Safe Bulk Actions
In list-only layout on desktop-sized screens, the Web UI SHALL allow selecting multiple jobs and performing safe bulk actions:
- **Run now**
- **Archive**
- **Unarchive**

The UI SHALL show a selection toolbar when one or more jobs are selected.

#### Scenario: Bulk run now skips archived jobs
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the user selects multiple jobs including at least one archived job
- **WHEN** the user triggers bulk Run now
- **THEN** the UI skips archived jobs
- **AND** the UI reports a summary outcome (queued/rejected/skipped/failed)

#### Scenario: Bulk archive requires confirmation
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the user selects one or more non-archived jobs
- **WHEN** the user triggers bulk Archive
- **THEN** the UI asks for confirmation before archiving
- **AND** the UI provides an option to cascade snapshot archival when supported

### Requirement: Jobs Table View Improves Sorting and Column Affordances
In jobs list-only Table view on desktop-sized screens, the Web UI SHALL:
- support header click sorting for at least job name and updated time,
- keep the header sort state and the sort control in sync, and
- keep key columns (at least name and actions) visible while horizontally scrolling.

#### Scenario: User sorts by clicking a table header
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** the jobs list is displayed in Table view
- **WHEN** the user clicks the Name column header to sort
- **THEN** the jobs list order updates
- **AND** the sort control reflects the same sort key and direction

### Requirement: Jobs Split View List Pane Is Resizable
On desktop-sized screens in split layout, the Web UI SHALL allow resizing the jobs list pane width via a drag handle.

The resized width SHALL be persisted on desktop-sized screens (local-only preference).

#### Scenario: User resizes the list pane and the width persists
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in split layout
- **WHEN** the user drags the split handle to resize the list pane
- **THEN** the list pane width updates immediately
- **AND** when the user reloads the page on desktop, the list pane uses the last selected width

### Requirement: Jobs List View Provides Quick Per-Row Actions
In Jobs list view on desktop-sized screens, the UI SHALL provide compact per-row actions without requiring opening the detail pane, including at least:
- Run now
- Edit
- More actions (overflow menu)

These actions SHOULD be visually de-emphasized until hover/focus to preserve scanability.

#### Scenario: Hover reveals per-row quick actions
- **GIVEN** the user is on a desktop-sized screen
- **AND** the jobs list is displayed in List view
- **WHEN** the user hovers a row (or focuses it via keyboard)
- **THEN** the UI reveals quick per-row actions for that row

### Requirement: Jobs List-only Layout Provides Clear Detail Access
When the Jobs workspace is in list-only layout and a job is selected, the UI SHALL provide a clear affordance to open the selected job in detail-only (or split) layout.

#### Scenario: User opens detail-only from list-only
- **GIVEN** the user is on a desktop-sized screen
- **AND** the Jobs workspace is in list-only layout
- **AND** a job is selected
- **WHEN** the user activates the "open details" affordance
- **THEN** the UI switches to a layout where job detail is visible
- **AND** the selected job remains selected

### Requirement: Mobile Job Detail Provides Sticky Actions
On mobile-sized screens, job detail actions SHALL be available without requiring the user to scroll back to the top of the page.

#### Scenario: User can run now while scrolled on mobile
- **GIVEN** the user is on a mobile-sized screen
- **AND** the user is viewing a job detail page
- **AND** the user has scrolled within the job detail content
- **WHEN** the user triggers Run now from the sticky actions area
- **THEN** the job run is triggered without requiring scrolling to the top

### Requirement: Jobs Workspace Filter Controls Have Accessible Names
The jobs list search input and filter controls SHALL provide stable accessible names (e.g. `name` attribute or `aria-label`) so they can be targeted reliably by automation and are not treated as unnamed form fields.

#### Scenario: Search and filters provide stable names
- **GIVEN** the user is on the Jobs workspace
- **WHEN** the UI renders the jobs list search and filter controls
- **THEN** each control provides a stable accessible name

### Requirement: Preset Theme Selection (6 Themes)
The Web UI SHALL provide a preset theme system that allows the user to choose one of **six** curated color schemes.

#### Scenario: User selects a different theme
- **GIVEN** the user opens Settings → Appearance
- **WHEN** the user selects a theme preset
- **THEN** the UI updates immediately (colors, surfaces, and background aurora)
- **AND** the selection persists across page reloads

### Requirement: Default Theme Is Mint Teal (Fresh + Bright)
The Web UI SHALL ship with **Mint Teal** as the default theme, targeting a fresh and bright look:
- light mode uses a mint-tinted page background with clean white content surfaces,
- teal is used as the primary action/selection accent,
- dark mode uses higher-contrast "blacker black / whiter white" surfaces and text (avoiding muddy gray haze).

#### Scenario: Fresh default palette is applied on first load
- **GIVEN** the user has no stored theme preference
- **WHEN** the Web UI loads
- **THEN** Mint Teal is applied automatically
- **AND** the UI remains readable and visually consistent in both light and dark mode

### Requirement: Higher-Contrast Dark Mode Across Themes
The Web UI SHALL improve dark mode perceived quality across all theme presets by providing:
- darker, cleaner base surfaces (reduced gray haze),
- whiter primary text for readability,
- and subtle aurora backgrounds that do not reduce legibility.

#### Scenario: Dark mode remains crisp and readable in every theme
- **GIVEN** the user enables dark mode
- **WHEN** the user switches between theme presets
- **THEN** text, cards, and borders maintain clear contrast
- **AND** the overall palette does not appear muddy or low-contrast

### Requirement: Theme-Specific Background Aurora
Each theme preset SHALL be able to define its own background aurora/gradient layers to reinforce the theme personality, while keeping the solid base background color separate.

#### Scenario: Background aurora does not reduce readability
- **WHEN** a theme is applied
- **THEN** the page background renders as `solid base + subtle aurora layers`
- **AND** text and primary content surfaces remain legible on both desktop and mobile

### Requirement: No Custom Theme Editing (This Iteration)
The Web UI SHALL NOT provide user-defined custom color editing in this iteration; only the preset themes are selectable.

#### Scenario: User cannot enter arbitrary colors
- **WHEN** the user opens Settings → Appearance
- **THEN** the UI offers only predefined theme choices
- **AND** there is no custom color picker or free-form input

### Requirement: Naive UI Theme Overrides Track Active Theme
The Web UI SHALL ensure that Naive UI theme overrides update when the active theme changes and SHALL avoid passing CSS `var(...)` strings into overrides.

#### Scenario: Theme switch updates component palette safely
- **WHEN** the user switches between themes
- **THEN** Naive UI components reflect the new colors immediately
- **AND** no runtime color parsing errors occur due to `var(...)` strings

### Requirement: Browser Theme Color Reflects Active Theme
The Web UI SHALL update `meta[name="theme-color"]` to reflect the active theme:
- in light mode it SHOULD use the theme’s primary accent color,
- in dark mode it SHOULD use the theme’s solid background color.

#### Scenario: Mobile browser chrome matches theme
- **GIVEN** the user is on a mobile browser
- **WHEN** the user toggles light/dark mode or switches theme presets
- **THEN** the browser chrome color updates to match the active theme intent

### Requirement: Jobs List Shows Latest Run Status And Time
In the Jobs workspace, the jobs list SHALL display each job's latest run status and latest run time (or an explicit empty state) to improve scanability.

#### Scenario: Desktop list shows status and time without increasing navigation
- **GIVEN** the user is on a desktop-sized screen
- **WHEN** the jobs list is displayed
- **THEN** each job row shows the latest run status (success/failed/running/queued/rejected as applicable)
- **AND** each job row shows the latest run time (or a clear "never ran" indication)

#### Scenario: Mobile list remains readable
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the jobs list is displayed
- **THEN** status and time are shown in a compact layout that does not force horizontal scrolling

### Requirement: Overview Shows Run Policy Strip
In the Jobs workspace Overview section, the UI SHALL show a compact run policy strip that includes schedule, schedule timezone, and overlap policy.

#### Scenario: Policy is visible without opening the editor
- **GIVEN** the user is viewing a job Overview
- **WHEN** the job has schedule configuration
- **THEN** the Overview shows schedule and timezone in the policy strip
- **AND** the Overview shows the overlap policy in the policy strip

#### Scenario: Policy strip wraps on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the Overview is rendered
- **THEN** the policy strip wraps naturally and remains fully usable

### Requirement: Overview Uses Compact Metadata Cards With Prominent Values
In the Jobs workspace Overview section, the UI SHALL present configuration metadata cards in a compact format while making the primary values visually prominent.

The cards SHALL include at least:
- source type,
- target type,
- backup format, and
- encryption.

#### Scenario: Values are emphasized while preserving vertical density
- **GIVEN** the user is viewing a job Overview
- **WHEN** metadata cards are rendered
- **THEN** labels are visually secondary
- **AND** values are visually prominent (larger typography and/or stronger emphasis)
- **AND** the cards do not waste vertical space

### Requirement: Format And Encryption Are Presented With Friendly Labels
In the Jobs workspace Overview section, backup format and encryption SHALL be presented using user-friendly labels, with optional code details where helpful.

#### Scenario: Format label and code are both available
- **GIVEN** the job uses a supported backup format
- **WHEN** the Overview renders the format card
- **THEN** the UI shows a friendly format label
- **AND** the UI MAY show the underlying format code as a secondary detail

#### Scenario: Encryption status is explicit
- **GIVEN** the job supports encryption
- **WHEN** the Overview renders the encryption card
- **THEN** the UI clearly indicates whether encryption is enabled or disabled
- **AND** when enabled, the UI shows which encryption key is selected as a secondary detail

### Requirement: History Provides Quick Status Filters
In the Jobs workspace History section, the UI SHALL provide quick status filter chips to allow narrowing run history without requiring a separate filter form.

#### Scenario: User filters to failures quickly
- **GIVEN** the user is viewing job History
- **WHEN** the user selects the "Failed" filter chip
- **THEN** the runs list shows only failed runs

#### Scenario: Filters remain usable on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the History filters are rendered
- **THEN** filter chips remain usable without horizontal overflow

### Requirement: Data Section Provides Guardrails For Destructive Actions
In the Jobs workspace Data section, the UI SHALL provide clear guardrails for destructive actions (such as retention apply and bulk delete) by showing warning text and scope hints near the actions.

#### Scenario: Retention action includes a warning and scope hint
- **GIVEN** the user is viewing the job Data section
- **WHEN** retention actions are available
- **THEN** the UI shows warning text describing impact (deleting snapshots) near the retention actions
- **AND** the UI indicates the scope is limited to the current job

#### Scenario: Bulk delete action includes a warning and scope hint
- **GIVEN** the user has selected snapshots to delete
- **WHEN** the delete action is available
- **THEN** the UI shows warning text describing impact (permanent deletion) near the delete action
- **AND** the UI indicates the scope is limited to the selected snapshots for the current job

### Requirement: Workbench Scroll Containers Provide Scrollability Cues
In the Jobs workspace on desktop-sized screens, scroll containers inside the workbench SHALL provide subtle cues (such as shadows/fades) indicating scrollability and scroll position.

#### Scenario: Jobs list pane shows scroll cues when overflowing
- **GIVEN** the jobs list pane content exceeds the pane height
- **WHEN** the user scrolls the jobs list pane
- **THEN** the UI shows subtle cues indicating additional content above/below

#### Scenario: Job content pane shows scroll cues when overflowing
- **GIVEN** the job content pane exceeds the pane height
- **WHEN** the user scrolls the job content pane
- **THEN** the UI shows subtle cues indicating additional content above/below

### Requirement: Jobs Overview Shows Configuration Metadata Cards
In the Jobs workspace Overview section, the Web UI SHALL present the job's key configuration metadata as compact summary cards.

The metadata SHALL include at least:
- source type,
- target type,
- backup format, and
- encryption.

#### Scenario: Overview displays configuration metadata with visual encoding
- **GIVEN** the user is viewing a job Overview
- **WHEN** the job has a defined spec
- **THEN** the UI shows cards for source type, target type, backup format, and encryption
- **AND** each card uses tags and/or text color to make the values scannable

#### Scenario: Cards remain usable on mobile
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the Overview is rendered
- **THEN** the metadata cards stack without horizontal overflow

### Requirement: Overview Does Not Provide Quick Link Shortcuts
The Jobs workspace Overview section SHALL NOT provide a dedicated “Quick links” block for navigating to History/Data.

#### Scenario: Navigation relies on section tabs
- **GIVEN** the user is viewing a job workspace
- **WHEN** the user wants to access History or Data
- **THEN** the user uses the job section navigation (Overview/History/Data)

### Requirement: Overview Shows Run Summary (Default Last 7 Days)
For a selected job, the Web UI SHALL show a run summary in the Overview section that defaults to the last 7 days.

#### Scenario: Overview shows latest run and 7-day counts
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/overview`
- **WHEN** run data exists for the job
- **THEN** the UI shows the latest run status and timestamp
- **AND** the UI shows compact run counts for the last 7 days (total, success, failed)
- **AND** the UI provides an action to open the latest run in the Run Detail drawer

#### Scenario: Overview handles jobs with no recent runs
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/overview`
- **WHEN** the job has no runs in the last 7 days
- **THEN** the UI shows a compact empty/zero state for the 7-day summary
- **AND** the UI does not offer a broken action to open a non-existent latest run

### Requirement: History Section Uses A Compact Header For Actions
The History section SHALL prioritize the runs list and SHALL place section actions (e.g. Refresh) in a compact header area rather than a standalone full-width action row.

#### Scenario: History actions do not consume a separate row
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/history`
- **WHEN** the History section is rendered
- **THEN** the runs list is shown as the primary content
- **AND** History actions are presented in the list panel header area
- **AND** the UI does not reserve a separate action row solely for a Refresh button

### Requirement: Data Section Uses Compact Per-Panel Actions
The Data section SHALL place actions for retention and snapshots inside their respective panel headers to reduce vertical space, especially on mobile.

#### Scenario: Retention save action is in the retention panel header
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/data`
- **WHEN** the retention panel is rendered
- **THEN** the primary Retention action (Save) is available in the retention panel header

#### Scenario: Snapshots refresh action is in the snapshots panel header
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId/data`
- **WHEN** the snapshots list is rendered
- **THEN** the snapshots Refresh action is available in the snapshots panel header

### Requirement: Mobile Toolbars Avoid Multi-Line Action Rows
On mobile-sized screens, job section actions (History/Data) SHALL avoid layouts that introduce additional standalone action rows or multi-line toolbars.

#### Scenario: Mobile shows compact actions without wrapping
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user views History or Data
- **THEN** actions are presented as compact icon/overflow controls in headers
- **AND** the UI avoids adding a separate action row that pushes primary content below the fold

### Requirement: Modern Colorful Theme Tokens
The Web UI SHALL define a richer set of shared design tokens (light + dark) for background/surfaces/text/accent/status colors so the UI looks modern, fresh, and consistent.

#### Scenario: A single token update changes the accent color across the UI
- **WHEN** the primary accent color token is updated
- **THEN** buttons, links, active navigation states, and primary highlights consistently reflect the updated accent color

### Requirement: Clear Surface Hierarchy With Reduced Visual Weight
The Web UI SHALL present a clear surface hierarchy (page background -> surfaces -> elevated panels) and SHALL reduce the heavy/industrial look by avoiding excessive borders and blur effects.

#### Scenario: Content surfaces are readable without heavy borders
- **WHEN** a standard content page (e.g. Agents or Jobs) is rendered
- **THEN** primary content is placed on solid surfaces (cards/panels)
- **AND** glass/blur is limited to navigation chrome, not the main content area

### Requirement: Refreshed Navigation Chrome With Strong Active State
The Web UI navigation chrome (desktop sider + header) SHALL adopt the refreshed theme and SHALL make the active section obvious using modern color cues (tint/indicator) rather than heavy borders.

#### Scenario: Active navigation item is clearly visible
- **WHEN** the user navigates between Dashboard, Jobs, and Agents
- **THEN** the active menu item is visually distinct via the shared theme (e.g. tint/indicator)

### Requirement: Consistent Page Header Action Layout
The Web UI SHALL standardize page header layout so that actions are easy to scan and do not overflow on mobile.

#### Scenario: Header actions collapse on mobile instead of wrapping
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the page header has multiple actions
- **THEN** non-critical actions are presented via an overflow menu instead of wrapping into multiple rows

### Requirement: Action Hierarchy and Safer Destructive Actions
The Web UI SHALL enforce a consistent action hierarchy:
- each page has at most one primary action,
- secondary actions are grouped,
- destructive actions are placed behind an overflow menu and require consistent confirmation.

#### Scenario: Destructive actions are not the most prominent controls
- **WHEN** a list row includes a destructive operation (e.g. revoke agent, delete)
- **THEN** the destructive operation is not a primary button in the row
- **AND** it requires an explicit confirmation step

### Requirement: Standard ListToolbar Pattern
The Web UI SHALL provide a shared ListToolbar pattern for list screens to support:
- search,
- filters,
- sort,
- view toggle (table/cards when applicable),
- refresh,
- and a primary action.

#### Scenario: List pages share the same toolbar layout
- **WHEN** the user opens Agents, Jobs, or Snapshots
- **THEN** each page presents the same core toolbar pattern (with page-specific controls) in a consistent location and layout

### Requirement: Standard SelectionToolbar for Bulk Actions
When list selection is available, the Web UI SHALL show a SelectionToolbar when one or more items are selected.

#### Scenario: Selecting rows reveals bulk actions
- **WHEN** the user selects one or more rows in a list
- **THEN** a selection toolbar appears showing the selected count
- **AND** bulk actions are discoverable without scrolling back to the page header

### Requirement: Agents List Efficiency Improvements
The Agents page SHALL improve efficiency by providing:
- search (name/id),
- quick status filters (online/offline/revoked),
- and a simplified per-row action set with overflow for secondary/danger actions.

#### Scenario: Operator finds an offline agent quickly
- **WHEN** the operator filters for offline agents and searches by name/id
- **THEN** the list updates immediately to show matching agents
- **AND** the operator can open details or agent-scoped pages with minimal clicks

### Requirement: Jobs List Efficiency Improvements
The Jobs list SHALL improve efficiency by providing:
- search (job name),
- filters (archived/type as applicable),
- sort (e.g. updated time/name),
- and consistent primary/secondary actions.

#### Scenario: Archived jobs are easy to find without visual clutter
- **WHEN** the user toggles "show archived" and searches by job name
- **THEN** archived jobs are clearly indicated
- **AND** primary actions that should not apply (e.g. Run now) are not presented as enabled

### Requirement: Snapshots List Efficiency Improvements
The Snapshots page SHALL improve efficiency by:
- supporting cursor-based pagination ("Load more"),
- providing filters (status/pinned/target),
- and making pinned snapshots visually distinct and protected from accidental deletion.

#### Scenario: Pinned snapshots are protected during bulk delete
- **WHEN** the user opens the bulk delete confirmation
- **THEN** pinned snapshots are clearly highlighted
- **AND** deletion requires an explicit extra acknowledgement when pinned items are included

### Requirement: Dashboard Health Summary and Actionable Links
The Dashboard SHALL act as a status center by showing a top-level health summary with actionable links to remediation screens.

#### Scenario: Operator can navigate from a health card to the relevant screen
- **WHEN** the Dashboard shows an offline agents or notification failures indicator
- **THEN** the indicator provides a direct navigation path to the relevant page with appropriate filters applied

### Requirement: Node Context Semantics Are Clear
The Web UI SHALL clearly communicate node context so users understand whether they are viewing global pages or node-scoped pages.

#### Scenario: Node picker behavior is clear on global pages
- **WHEN** the user changes the node selection while on a global page (e.g. Dashboard)
- **THEN** the UI communicates that it updates the preferred node (used for future node-scoped navigation)
- **AND** it does not silently change the current page scope unless the page is node-scoped

### Requirement: Visual Accessibility Remains Intact After Refresh
The refreshed visual design SHALL preserve focus-visible indicators, SHOULD maintain sufficient color contrast in light/dark mode, and SHOULD respect reduced motion preferences.

#### Scenario: Keyboard focus remains clearly visible after theme changes
- **WHEN** the user navigates the UI with the keyboard
- **THEN** the focused control shows a clearly visible focus indicator under the refreshed theme

### Requirement: Job Detail actions toolbar
The Web UI SHALL provide a job-level actions toolbar on the Job Detail page (`/n/:nodeId/jobs/:jobId`) so that common actions are accessible without switching to a secondary tab.

#### Scenario: User sees common actions on Job Detail
- **GIVEN** the user is on `/n/:nodeId/jobs/:jobId`
- **THEN** the UI shows a toolbar with job-level actions (run now, edit, deploy, archive/unarchive, delete)

#### Scenario: Destructive actions require confirmation
- **WHEN** the user attempts to archive or delete a job from the toolbar
- **THEN** the UI requires explicit confirmation before performing the action

### Requirement: Dashboard Overview Page (Metrics + Trend + Recent Runs)
The Web UI SHALL provide a Dashboard page that surfaces a high-level overview of backup status and recent activity.

#### Scenario: Dashboard shows overview (no checklist)
- **WHEN** the user opens the Dashboard page
- **THEN** the page shows overview sections (stats, trend, recent runs)
- **AND** the page does not show a setup checklist

#### Scenario: Dashboard works when there is no data yet
- **GIVEN** there are no runs yet
- **WHEN** the user opens the Dashboard page
- **THEN** the Dashboard shows zero/empty values without errors
- **AND** the recent runs section displays an empty-state message

#### Scenario: Recent runs list links to Run Detail
- **GIVEN** there is a run in the recent runs list
- **WHEN** the user clicks it
- **THEN** the UI navigates to the Run Detail page for that run

### Requirement: Node-Scoped Job Detail Page
The Web UI SHALL provide a node-scoped Job Detail page for a specific job at `/n/:nodeId/jobs/:jobId`.

#### Scenario: Job Detail is accessible from Jobs list
- **GIVEN** the user is on `/n/:nodeId/jobs`
- **WHEN** the user opens a job
- **THEN** the UI navigates to `/n/:nodeId/jobs/:jobId`

#### Scenario: Job Detail provides Runs and Snapshots views
- **WHEN** the user is on the Job Detail page
- **THEN** the user can view job runs
- **AND** the user can view job snapshots

#### Scenario: Runs list links to Run Detail
- **GIVEN** the job has a run in the Runs tab
- **WHEN** the user clicks the run
- **THEN** the UI navigates to `/n/:nodeId/runs/:runId`

### Requirement: Jobs List Action Simplification
The Jobs list SHALL prioritize primary actions and move secondary actions into a compact overflow menu.

#### Scenario: Jobs list keeps primary actions visible
- **WHEN** the user views the Jobs list
- **THEN** the primary action “Run now” is visible
- **AND** secondary actions (edit, deploy, archive/delete) are available via a “More” menu

### Requirement: Bulk Operation Detail Auto-Refresh
The Web UI SHALL automatically refresh the Bulk Operation detail view while an operation is running.

#### Scenario: Running operation refreshes without manual action
- **GIVEN** a bulk operation is in `running` status
- **WHEN** the user opens the operation detail view
- **THEN** the UI refreshes the operation detail automatically until it is no longer running

### Requirement: Failure-Focused Filtering
The Web UI SHALL allow focusing on failed items in a bulk operation.

#### Scenario: User filters to failed items
- **GIVEN** a bulk operation has failed items
- **WHEN** the user enables a “failed only” filter
- **THEN** the item list shows only failed items

### Requirement: Agents Page Quick Links to Node Context
The Agents page SHALL provide quick navigation to common node-scoped pages for a given agent.

#### Scenario: Jump to agent Jobs
- **GIVEN** an agent is listed on the Agents page
- **WHEN** the user clicks “Jobs”
- **THEN** the UI navigates to `/n/:agentId/jobs`

#### Scenario: Jump to agent Storage
- **GIVEN** an agent is listed on the Agents page
- **WHEN** the user clicks “Storage”
- **THEN** the UI navigates to `/n/:agentId/settings/storage`

### Requirement: Enrollment Token Provides Command Template
The enrollment token modal SHALL display a copyable CLI command template.

#### Scenario: Token modal shows enroll command
- **GIVEN** an enrollment token is created
- **THEN** the UI shows a copyable command template containing:
  - the Hub URL
  - the enroll token value
  - placeholders for agent name

### Requirement: Preferred Node for Node-Scoped Navigation
The Web UI SHALL maintain a preferred node selection used as the default node when navigating to node-scoped pages from global routes.

#### Scenario: Selecting a node on a global page does not change the route
- **GIVEN** the user is on a global page (e.g. `/`)
- **WHEN** the user changes the node selector
- **THEN** the UI does not navigate away from the current page
- **AND** the preferred node is updated for subsequent node-scoped navigation

#### Scenario: Jobs navigation uses preferred node when not already in a node scope
- **GIVEN** the user is on a global page
- **AND** the preferred node is set to an agent id `agent1`
- **WHEN** the user navigates to Jobs
- **THEN** the UI navigates to `/n/agent1/jobs`

### Requirement: Node Context Cue in Page Headers
The Web UI SHALL display a clear node context cue on node-scoped pages.

#### Scenario: Jobs page shows node context
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the page header shows the selected node context (Hub or Agent)

### Requirement: UI Provides A Help Entry Point To Product Docs
The Web UI SHALL provide a visible "Help" entry point that opens the product documentation at `/docs/`.

#### Scenario: User opens docs from the header menu
- **WHEN** the user clicks "Help"
- **THEN** the browser navigates to `/docs/`

### Requirement: Run Detail Shows Stage Timeline and Durations
The Run Detail page SHALL present a stage timeline for Scan / Build / Upload with durations.

#### Scenario: Completed run shows per-stage durations
- **GIVEN** a run has ended
- **THEN** the UI shows durations for Scan, Build, and Upload when those stages occurred
- **AND** the UI shows total duration

#### Scenario: Running run shows partial stage timing
- **GIVEN** a run is still running
- **THEN** the UI shows elapsed time for the current stage

### Requirement: Run Detail Preserves Transfer Metrics After Completion
The Run Detail page SHALL preserve meaningful transfer metrics after a run completes.

#### Scenario: Completed run shows final average transfer rate
- **GIVEN** a run has ended
- **THEN** the UI shows a final transfer rate value instead of replacing it with a placeholder

### Requirement: Run Detail Indicates Failure Stage
The Run Detail page SHALL indicate the stage in which a run failed when the information can be determined.

#### Scenario: Failed run shows failure stage
- **GIVEN** a run ends with status `failed` or `rejected`
- **WHEN** the UI can determine the last active stage
- **THEN** the UI displays that stage as the failure stage

### Requirement: Run Detail Shows A Progress Panel
The Web UI SHALL render a dedicated Progress panel on the node-scoped Run Detail page, replacing the single-line progress text.

#### Scenario: User can read overall progress at a glance
- **GIVEN** a user opens a running backup run on Run Detail
- **WHEN** the UI loads the latest progress snapshot
- **THEN** the Progress panel shows an overall progress bar and key stats without requiring the user to parse a single long text line

### Requirement: Progress Panel Shows Stage Breakdown With Help
The Progress panel SHALL show the backup stages (Scan, Packaging, Upload) with per-stage progress and a help entrypoint ("?") explaining each stage.

#### Scenario: User opens packaging stage help
- **GIVEN** a backup run is in the packaging stage
- **WHEN** the user clicks the "?" help entrypoint for Packaging
- **THEN** the UI shows a short explanation of what Packaging is doing for the selected backup format

### Requirement: Progress Panel Is Mobile-Friendly
The Progress panel SHALL adapt to mobile screens using a stacked layout and collapsible sections while preserving readability.

#### Scenario: Progress panel remains usable on mobile
- **GIVEN** a user opens Run Detail on a small screen
- **WHEN** the Progress panel is displayed
- **THEN** key progress information remains visible without requiring horizontal scrolling

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: User opens Run Detail from the runs list
- **GIVEN** a user is viewing a job’s run list
- **WHEN** the user selects a run
- **THEN** the UI navigates to the Run Detail page for that run

#### Scenario: Run Detail presents a clear “status + key facts + actions” header
- **GIVEN** a run detail is loaded
- **THEN** the page shows the run status as a prominent badge near the title
- **AND** the run id is displayed as secondary information with a copy affordance
- **AND** primary actions are visually separated from secondary actions

### Requirement: Run Detail Shows Events and Linked Operations
The Run Detail page SHALL show live run events and a sub-list of linked operations (restore/verify) started from the run.

#### Scenario: Restore operation remains visible after closing dialogs
- **WHEN** a user starts a restore from Run Detail
- **THEN** the resulting restore operation appears in the operations sub-list for that run

#### Scenario: Operations empty state is compact and readable
- **GIVEN** a run has no linked operations
- **THEN** the operations section shows a compact empty state
- **AND** it does not render a large empty table that dominates the page

### Requirement: Filesystem Job Editor Supports Pre-Scan Toggle
The Web UI SHALL expose a filesystem job option to enable/disable pre-scan (`source.pre_scan`) and default it to enabled for new jobs.

#### Scenario: New filesystem job defaults pre-scan on
- **WHEN** the user opens the create-job dialog for a filesystem job
- **THEN** the pre-scan option is enabled by default

#### Scenario: User disables pre-scan
- **WHEN** the user disables pre-scan and saves the job
- **THEN** the saved job spec includes `source.pre_scan = false`

### Requirement: WebDAV Prefix Can Be Picked via Browser Modal
The Web UI SHALL allow browsing and selecting a WebDAV destination prefix using the shared picker modal UX.

#### Scenario: Select a destination prefix
- **WHEN** the user clicks “Browse” for a WebDAV destination prefix
- **THEN** the UI opens a picker that lists WebDAV directories and allows selecting the current directory as the prefix

### Requirement: Path Picker Is Data-Source Driven With Capability-Gated UI
The web UI SHALL provide a path picker implementation that is driven by a data source interface and a capability declaration so that new storage backends can reuse the picker UI without duplicating behavior.

#### Scenario: Filesystem browsing uses the generic picker
- **GIVEN** a filesystem data source backed by the existing filesystem list endpoint
- **WHEN** the user opens the filesystem picker
- **THEN** the picker uses the generic data-source driven path picker implementation

#### Scenario: Unsupported features are hidden or disabled
- **GIVEN** a data source that does not support a specific filter/sort/column
- **WHEN** the picker renders the filter and table UI
- **THEN** unsupported controls or columns are not shown (or are disabled) to prevent invalid requests

#### Scenario: A new storage backend can reuse the picker UI
- **GIVEN** a future WebDAV/S3 data source that implements the picker data source interface
- **WHEN** the UI adds a browser for that backend
- **THEN** the UI reuses the generic picker without rewriting the picker UI layout and interaction patterns

### Requirement: Picker Tables Have Clear Yet Subtle Visual Hierarchy
The picker tables SHALL use a clear but subtle visual hierarchy to improve readability on both desktop and mobile.

#### Scenario: Hover and selection are visually distinguishable
- **GIVEN** a picker modal renders a list of entries
- **WHEN** the user hovers a row or selects it
- **THEN** the hover/selected state is visually distinguishable without making the table look overly heavy

### Requirement: Filesystem Picker Supports Sorting
The web UI filesystem picker SHALL allow sorting by name, modified time, and size, while preserving stable pagination.

#### Scenario: User sorts by modified time descending
- **GIVEN** the filesystem picker is open for a directory
- **WHEN** the user chooses sorting by “modified time” in descending order
- **THEN** the picker refreshes from the first page using server-side sorting
- **AND** the displayed sort state matches the active sort selection

### Requirement: Filesystem Picker Explains Empty and Error States
The filesystem picker SHALL clearly distinguish empty states and common error states, and provide contextual recovery actions.

#### Scenario: Empty directory vs no matches
- **GIVEN** the filesystem picker is open
- **WHEN** no filters/search are active and the directory contains no entries
- **THEN** the UI shows an “empty directory” state
- **WHEN** filters/search are active and the filtered result is empty
- **THEN** the UI shows a “no matches” state and suggests clearing filters

#### Scenario: Agent offline recovery
- **GIVEN** the user is browsing a non-Hub node
- **WHEN** the agent is offline
- **THEN** the UI shows an “agent offline” error state
- **AND** provides a “retry” action

### Requirement: Filesystem Picker Persists Per-Node Filters
The filesystem picker SHALL persist per-node filter state and restore it when the picker is opened again for the same node.

#### Scenario: Filters are restored on next open
- **GIVEN** the user applied filters in the filesystem picker for a node
- **WHEN** the user closes and reopens the picker for the same node
- **THEN** the previously applied filters are restored

### Requirement: Picker Modals Provide Selection Helpers
The web UI picker modals SHALL provide selection helpers for efficient bulk selection in a paged table.

#### Scenario: Select a range with Shift
- **GIVEN** a picker modal displays a list of entries
- **WHEN** the user selects one row and shift-selects another row
- **THEN** all rows in the range become selected

#### Scenario: Select all loaded rows
- **GIVEN** a picker modal shows a paged listing
- **WHEN** the user clicks “Select all”
- **THEN** all currently loaded rows are selected
- **AND** the UI indicates that selection applies to loaded rows (not the entire directory)

### Requirement: Picker Modals Support Keyboard Shortcuts
The web UI picker modals SHALL support keyboard shortcuts for common navigation actions.

#### Scenario: Navigate with keyboard
- **GIVEN** a picker modal is open
- **WHEN** the user presses `Backspace` while not typing in an input
- **THEN** the picker navigates to the parent directory/prefix
- **WHEN** the user presses `Ctrl/Cmd+L`
- **THEN** the path/prefix editor receives focus
- **WHEN** the user presses `Esc`
- **THEN** the modal closes

### Requirement: Picker Modals Provide Accessible Labels
The web UI picker modals SHALL provide accessible labels and predictable focus order for icon-only controls.

#### Scenario: Icon-only actions have an accessible label
- **GIVEN** a picker modal renders icon-only controls (e.g., refresh, up)
- **WHEN** the controls are focused
- **THEN** they expose an accessible name via `aria-label` (and/or `title` as a fallback)

### Requirement: Picker Modals Use Shared Layout Building Blocks
The web UI SHALL implement picker modals using shared layout building blocks to reduce duplication and keep UX consistent across pickers.

#### Scenario: A shared layout prevents “fix one modal, forget the other”
- **GIVEN** the filesystem picker and restore entries picker share common UX elements (search, filters, table, footer)
- **WHEN** a UX fix is applied to the shared picker layout
- **THEN** both pickers reflect the fix without requiring duplicated changes

### Requirement: Jobs UI Provides “Deploy to Nodes” Action
The Web UI SHALL provide a “Deploy to nodes” action for an existing job.

#### Scenario: Operator starts deploy from job list
- **WHEN** the operator triggers “Deploy to nodes” for a job
- **THEN** the UI MUST open a flow to select target nodes and configure naming

### Requirement: UI Supports Label-based Selection and Naming Template
The deploy flow SHALL allow selecting nodes via labels (AND/OR) and SHALL provide a naming template input with a sensible default that includes the node id.

#### Scenario: Label selector targets a subset
- **WHEN** the operator selects labels and AND/OR mode
- **THEN** the UI MUST target the resolved node set

### Requirement: UI Shows Preview and Validation Results
The UI SHALL show a preview of planned job names and per-node validation results before executing the deploy.

#### Scenario: Preview highlights failures
- **GIVEN** some nodes are missing prerequisites
- **WHEN** the operator views the preview
- **THEN** the UI MUST highlight which nodes will fail and why

### Requirement: UI Shows Execution Results Via Bulk Operations Panel
The UI SHALL show deploy execution progress and per-node outcomes via the bulk operations panel/page.

#### Scenario: Operator can retry failed nodes
- **GIVEN** some nodes failed during deploy
- **WHEN** the operator chooses to retry failed
- **THEN** the UI MUST re-run only failed nodes via the bulk operations framework

### Requirement: Agents Page Shows Config Sync Status
The Web UI SHALL show a quick, scannable config sync status indicator for each agent.

#### Scenario: Operator can identify out-of-sync nodes
- **WHEN** the user opens the Agents page
- **THEN** each agent row MUST display whether the agent is synced, pending, offline, or in error

### Requirement: Agent Details Show Desired/Applied Snapshot and Errors
The Web UI SHALL provide a detailed view that shows desired/applied snapshot ids and the latest sync error information.

#### Scenario: Operator inspects a node’s sync details
- **WHEN** the user opens an agent detail view
- **THEN** the UI MUST display desired/applied snapshot ids and last sync error (if any)

### Requirement: UI Supports “Sync Now” Actions
The Web UI SHALL provide “sync now” actions for single-node and bulk selections (via bulk operations UI).

#### Scenario: User triggers single-node sync now
- **WHEN** the user clicks “sync now” for a node
- **THEN** the UI MUST call the backend API and display success/error feedback

### Requirement: UI Provides Bulk Operations Panel
The Web UI SHALL provide a panel/page to view bulk operations and their per-node results.

#### Scenario: Operator can see progress and failures
- **WHEN** the user opens a bulk operation
- **THEN** the UI MUST show overall progress and per-node results
- **AND** MUST display error summaries for failed items

### Requirement: UI Supports Retry Failed and Cancel
The Web UI SHALL allow an operator to retry failed items and cancel an operation.

#### Scenario: Retry failed is available from UI
- **GIVEN** an operation has failed items
- **WHEN** the user clicks “retry failed”
- **THEN** the UI MUST trigger the backend retry API and refresh status

#### Scenario: Cancel is available from UI
- **WHEN** the user clicks “cancel”
- **THEN** the UI MUST trigger the backend cancel API and refresh status

### Requirement: Agents Page Can Start Bulk Label Updates
The Web UI SHALL provide an entry point on the Agents page to start bulk label operations (add/remove labels) using a node selector.

#### Scenario: User starts bulk label add
- **GIVEN** the user has selected a set of nodes (explicit selection or label filter)
- **WHEN** the user starts a bulk “add labels” operation
- **THEN** the UI MUST create a bulk operation and show its progress/results

### Requirement: Hub Runtime Config Page (Restart Required)
The Web UI SHALL provide a Hub-only runtime config page that supports viewing and editing selected configuration.

#### Scenario: User sees effective vs saved config
- **WHEN** the user opens the runtime config page
- **THEN** the page MUST display the current effective value for each field
- **AND** the saved (pending) value if present
- **AND** a clear indicator that changes require a restart to take effect

### Requirement: Read-only Display For Unsafe Fields
The Web UI SHALL display these fields as read-only:
- Bind host/port
- Trusted proxies
- Insecure HTTP

#### Scenario: Unsafe fields cannot be edited
- **WHEN** the user views the runtime config page
- **THEN** the UI MUST NOT allow editing of unsafe fields

### Requirement: Editable Fields For Safe Policy Settings
The Web UI SHALL allow editing of safe policy settings (persisted to DB and applied on restart):
- Hub timezone
- Run retention days
- Incomplete cleanup days
- Logging filter/file/rotation/keep-files

#### Scenario: Save prompts restart
- **WHEN** the user saves updated runtime config
- **THEN** the UI MUST confirm the save
- **AND** indicate a restart is required for the changes to take effect

### Requirement: Notifications Sub-Navigation Uses A Single Source Of Truth
The Web UI SHALL derive Notifications settings subpages (mobile list entries and desktop tabs) from a single shared config.

#### Scenario: Adding a new Notifications subpage updates all entry points
- **WHEN** a new Notifications subpage is added to the shared config
- **THEN** it appears in the Notifications index list
- **AND** it appears in the desktop Notifications tab bar

### Requirement: Tests Guard Against Notifications Nav/Router Drift
The Web UI SHALL include unit tests that fail if a configured Notifications subpage does not resolve in the router or has inconsistent metadata.

#### Scenario: Configured subpages always resolve
- **GIVEN** a Notifications subpage in the shared config
- **THEN** `router.resolve(to)` MUST match at least one route record

#### Scenario: Title keys stay consistent
- **GIVEN** a Notifications subpage in the shared config
- **THEN** its router route SHOULD use the same `meta.titleKey`

### Requirement: Language Options Use A Single Source Of Truth
The Web UI SHALL derive language dropdown options from `supportedLocales` via a single shared helper.

#### Scenario: A new locale becomes selectable everywhere
- **WHEN** a locale is added to `supportedLocales`
- **THEN** it is available in language dropdowns across the app

### Requirement: Tests Guard Against Locale Option Drift
The Web UI SHALL include unit tests that fail if a supported locale is missing a label or a Naive UI locale mapping.

#### Scenario: Supported locales are fully mapped
- **GIVEN** a locale in `supportedLocales`
- **THEN** it MUST have a dropdown label
- **AND** it MUST have Naive UI `locale` and `dateLocale` mappings

### Requirement: Settings Navigation Uses A Single Source Of Truth
The Web UI SHALL derive Settings navigation entries (overview list and sidebar submenu) from a single shared config.

#### Scenario: New settings section appears in both places
- **WHEN** a new Settings section is added to the shared Settings nav config
- **THEN** it is visible in the Settings overview list
- **AND** it is visible in the desktop Settings sidebar submenu

### Requirement: Tests Guard Against Overview/Submenu Drift
The Web UI SHALL include unit tests that fail if a Settings entry is visible in the overview list but missing from the sidebar submenu.

#### Scenario: Overview items are always included in submenu
- **GIVEN** a Settings nav entry is configured to show in the overview list
- **THEN** it MUST also be configured to show in the sidebar submenu

### Requirement: About Page Shows Hub And UI Build Info
The Web UI SHALL provide an authenticated About page that shows Hub and UI version and build time.

#### Scenario: About page is behind authentication
- **WHEN** an unauthenticated user navigates to the About page
- **THEN** the user is redirected to login

#### Scenario: About page shows build info
- **GIVEN** an authenticated user
- **WHEN** they open Settings -> About
- **THEN** the page shows Hub version + build time
- **AND** shows UI version + build time

### Requirement: Follow Auto-Resumes When Returning To Bottom
The Web UI SHALL automatically re-enable follow mode when the user scrolls back to the bottom after follow was auto-disabled by scrolling away.

#### Scenario: Auto-disabled follow resumes when reaching bottom
- **GIVEN** follow mode was enabled
- **AND** follow was auto-disabled because the user scrolled away from the bottom
- **WHEN** the user scrolls back to the bottom of the Run Events list
- **THEN** follow mode is automatically re-enabled

#### Scenario: Manually disabled follow does not auto-resume
- **GIVEN** follow mode was manually disabled via the follow switch
- **WHEN** the user scrolls to the bottom of the Run Events list
- **THEN** follow mode remains disabled

### Requirement: Run Events Are Rendered As A Log List With Compact Rows
The Web UI SHALL render Run Events as a log list optimized for scanning and long-running tasks.

#### Scenario: Desktop row is single-line and scannable
- **WHEN** the viewport is `>= md`
- **THEN** each Run Event row shows time + level + kind + up to 2 summary chips + message (ellipsis) + Details

#### Scenario: Mobile row is compact and readable
- **WHEN** the viewport is `< md`
- **THEN** each Run Event row uses a compact two-line layout without excessive row height growth

### Requirement: Run Events Show Up To Two High-Signal Summary Chips
The Web UI SHALL display up to 2 summary chips per event derived from `event.fields` to help users quickly interpret outcomes (attempts, error kind, next retry time, durations, counts, etc.).

#### Scenario: Only two chips are rendered
- **GIVEN** an event has more than 2 eligible summary fields
- **WHEN** the Run Events list renders the row
- **THEN** at most 2 chips are shown

#### Scenario: Retry scheduling uses relative time
- **GIVEN** an event includes `next_attempt_at`
- **WHEN** the Run Events list renders the row
- **THEN** the value is shown in relative time (e.g., minutes from now)

### Requirement: Follow/Tail Behavior Matches Common Log Viewers
The Web UI SHALL support a “follow latest” mode by default and SHALL avoid fighting the user when they scroll up.

#### Scenario: Follow turns off when user scrolls away
- **GIVEN** follow mode is enabled
- **WHEN** the user scrolls away from the bottom of the list
- **THEN** follow mode is disabled

#### Scenario: New events are counted while follow is off
- **GIVEN** follow mode is disabled
- **WHEN** new events arrive
- **THEN** the UI shows an incrementing “new events” count
- **AND** the user can click “Latest” to jump to bottom and re-enable follow

### Requirement: WS Auto-Reconnect Is Enabled By Default
The Web UI SHALL automatically reconnect the Run Events websocket when disconnected, and SHALL provide a manual reconnect action.

#### Scenario: Auto reconnect attempts are visible
- **WHEN** the websocket disconnects unexpectedly
- **THEN** the UI shows a reconnecting state and the countdown to the next attempt

### Requirement: Details View Is Optimized For Desktop And Mobile
The Web UI SHALL provide an event details view for full message and fields JSON.

#### Scenario: Desktop uses a modal detail view
- **WHEN** the viewport is `>= md`
- **THEN** details open in a modal

#### Scenario: Mobile uses a half-screen bottom drawer
- **WHEN** the viewport is `< md`
- **THEN** details open in a bottom drawer (~70vh)

### Requirement: Schedule Editor Supports Simple and Cron Modes
The Web UI SHALL provide a schedule editor for jobs that supports manual mode, a guided simple mode for common schedules, and an advanced cron expression mode.

#### Scenario: User selects “Daily at 02:00”
- **WHEN** a user configures “Daily at 02:00” in simple mode
- **THEN** the UI generates the corresponding cron and saves it

### Requirement: Schedule Timezone Is Configurable Per Job
The Web UI SHALL allow users to select a timezone for schedule interpretation, defaulting to the Hub timezone.

#### Scenario: Default timezone is hub timezone
- **GIVEN** the Hub timezone is `UTC`
- **WHEN** a user opens the job editor
- **THEN** the timezone selector defaults to `UTC`

### Requirement: UI Communicates DST Behavior
The Web UI SHALL clearly communicate that schedules are interpreted in the selected timezone and that DST gaps are skipped and DST folds run once.

#### Scenario: DST help text is visible
- **WHEN** a user views the schedule configuration section
- **THEN** the UI shows a brief explanation of DST behavior

### Requirement: UI Explains Cleanup Page Actions
The web UI SHALL provide an in-page help dialog that explains the effect of each cleanup page action button.

#### Scenario: User opens the help dialog and sees action explanations
- **GIVEN** the user is on the incomplete run cleanup page
- **WHEN** the user clicks the “?” help button
- **THEN** the UI shows short explanations for “更多”, “立即重试”, “忽略”, and “取消忽略”

### Requirement: Cleanup Page Uses Clear Title
The web UI SHALL label the cleanup page as “Incomplete run cleanup” (and a localized equivalent).

#### Scenario: Chinese locale shows a clear title
- **GIVEN** the UI locale is `zh-CN`
- **WHEN** the user opens the cleanup page
- **THEN** the page title is “不完整运行清理”

### Requirement: UI Explains Cleanup Task Statuses
The web UI SHALL provide an in-page help dialog that explains the meaning of each cleanup task status.

#### Scenario: User opens the status help dialog
- **WHEN** the user clicks the “?” help button on the cleanup page
- **THEN** the UI shows short explanations for `queued`, `running`, `retrying`, `blocked`, `done`, `ignored`, and `abandoned`

### Requirement: UI Provides Incomplete Cleanup Management (Mobile Friendly)
The web UI SHALL provide a mobile-friendly page to view incomplete cleanup tasks and take operator actions.

#### Scenario: Mobile layout is usable
- **WHEN** the user opens the cleanup page on a small screen
- **THEN** tasks are displayed in a card layout with readable status, error, and actions without horizontal scrolling

### Requirement: UI Supports Archive vs Permanent Delete
The web UI SHALL let users choose between archiving a job and permanently deleting it.

#### Scenario: Archive is available from the delete flow
- **WHEN** the user attempts to delete a job
- **THEN** the UI offers an “Archive (keep history)” option and a “Delete permanently (cascade)” option

### Requirement: Toast-Style Error Messages Use the Shared Error Formatter
For non-form actions and modal workflows that surface errors via toasts, the Web UI SHALL use the shared error formatter so that:
- Known backend `error` codes are localized.
- Unknown codes fall back to backend `message`.

This reduces regressions where UI code only shows a generic “failed” message or loses the backend error code.

#### Scenario: Toast displays localized known error code
- **WHEN** an API call fails and returns a known `error` code
- **THEN** the UI shows the localized message for that code

#### Scenario: Toast falls back to backend message for unknown code
- **WHEN** an API call fails and returns an unknown `error` code with a `message`
- **THEN** the UI shows the backend `message`

### Requirement: UI Captures Request ID and Surfaces It for Internal Errors
The Web UI SHALL capture `X-Request-Id` from API responses and attach it to the thrown API error object.

For 5xx/internal errors, the UI SHOULD surface the Request ID to help users correlate UI failures with server logs.

#### Scenario: Internal error includes a Request ID for troubleshooting
- **WHEN** an API call fails with HTTP 500 `internal_error`
- **AND** `X-Request-Id` is present in the response headers
- **THEN** the UI surfaces the Request ID to the user

### Requirement: Run Events Timestamp Is Non-Wrapping In Fixed Rows
The Web UI SHALL render the Run Events timestamp column as a single line (no wrapping) and SHALL provide sufficient vertical spacing so the timestamp does not visually collide with row borders.

#### Scenario: Timestamp does not wrap and stays readable
- **WHEN** the Run Events viewer displays events in a fixed-height virtual list row
- **THEN** the timestamp text remains on one line
- **AND** the timestamp is vertically comfortable (not touching borders)

### Requirement: Responsive Run Events Timestamp Format
The Web UI SHALL display a responsive timestamp format for Run Events:
- On desktop viewports (`>= md`): show a compact date+time format suitable for scanning.
- On mobile viewports (`< md`): show a concise time-only format (`HH:mm`).

#### Scenario: Desktop shows compact date+time
- **WHEN** the viewport is `>= md`
- **THEN** each Run Event row shows a compact date+time timestamp

#### Scenario: Mobile shows time-only
- **WHEN** the viewport is `< md`
- **THEN** each Run Event row shows a time-only timestamp in `HH:mm` format

### Requirement: Full Timestamp Remains Accessible
The Web UI SHALL ensure the full timestamp information remains accessible even when the list uses a compact format.

#### Scenario: Full timestamp can be viewed
- **WHEN** a user opens the Run Event details view
- **THEN** the full timestamp is visible in the details

### Requirement: Run Events WebSocket Uses `after_seq`
The Web UI SHALL connect the run events WebSocket with `after_seq` equal to the last known event sequence to avoid receiving duplicate catch-up events.

#### Scenario: No duplicate catch-up events after initial load
- **WHEN** the user opens the Run Events viewer and the UI has already loaded events up to sequence N
- **THEN** the WebSocket connection includes `after_seq = N`
- **AND** the UI does not process duplicated events for sequences `<= N`

### Requirement: Run Events Viewer Supports Large Event Counts
The Run Events viewer SHALL remain responsive for runs with large numbers of events.

#### Scenario: Viewer remains responsive with many events
- **WHEN** a run produces a large number of events
- **THEN** the UI uses an efficient rendering strategy (e.g., virtualization or fixed-height rows) to avoid rendering all events at once

### Requirement: Event Details Are Shown On Demand
The Run Events viewer SHALL avoid rendering large JSON payloads inline by default and SHALL provide an on-demand way to inspect event details (such as `fields`).

#### Scenario: Event fields are viewed in a details UI
- **WHEN** an event contains structured `fields`
- **THEN** the user can open a details view to inspect the JSON

### Requirement: Follow Mode Preserves User Scroll Position
The Run Events viewer SHALL support a follow mode (auto-scroll to latest events) that can be disabled to preserve the user’s scroll position while reading historical output.

#### Scenario: Follow mode disabled preserves scroll
- **WHEN** follow mode is disabled and new events arrive
- **THEN** the UI does not automatically scroll to the bottom

### Requirement: Targets UI Is Node-Scoped
In node context, the Web UI SHALL render Targets/Storage pages for the selected node and SHALL only show and edit targets within that node scope.

#### Scenario: Storage page shows only node targets
- **WHEN** the user opens `/n/<agent_id>/settings/storage`
- **THEN** only targets belonging to that Agent node are shown

### Requirement: Job Editor Enforces Node-Scoped Targets
In node context, the Web UI SHALL only allow selecting targets belonging to the selected node when creating/editing jobs.

#### Scenario: Job editor hides cross-node targets
- **WHEN** the user edits a job on `/n/hub/jobs`
- **THEN** the target selector shows only Hub-scoped targets

### Requirement: Node Context and Node-Scoped Routes
The Web UI SHALL support a first-class node context and SHALL encode it in node-scoped routes under `/n/:nodeId/**` so node context persists across refreshes and deep links.

`nodeId` MUST support:
- the local Hub node (reserved id `hub`), and
- enrolled Agent nodes (using their `agent_id`).

#### Scenario: Node context survives refresh
- **WHEN** a user opens `/n/hub/jobs` and refreshes the page
- **THEN** the UI remains in the Hub node context and the Jobs list is shown for the Hub node

### Requirement: Node Switcher
The Web UI SHALL provide a node switcher that allows selecting the Hub node and any enrolled Agent node.

The node switcher SHOULD display basic status (e.g. online/offline) for Agents.

#### Scenario: User switches to an Agent node
- **WHEN** a user selects an Agent node from the node switcher
- **THEN** the UI navigates to the equivalent node-scoped route under `/n/:nodeId/**`

### Requirement: Per-Node UX Matches Single-Node Behavior
In node context, the Web UI SHALL behave like a single-node app for the selected node:
- node-scoped lists (e.g. Jobs and their run history) are filtered to the selected node,
- create/edit defaults to the selected node,
- cross-node selection controls are hidden or disabled.

#### Scenario: Create job defaults to selected node
- **WHEN** the user is on `/n/<agent_id>/jobs` and opens the Create Job wizard
- **THEN** the job is created for that Agent node without requiring a separate node selection step

#### Scenario: Job run history is filtered by node context
- **WHEN** the user is on `/n/<agent_id>/jobs` and opens run history for a job
- **THEN** the run history shown belongs to jobs on that Agent node

### Requirement: Global Pages Are Clearly Separated
The Web UI SHALL keep global management pages outside node-scoped routes (e.g. Agents management, global notifications/settings).

#### Scenario: Agents page is global
- **WHEN** the user opens the Agents page
- **THEN** it shows all enrolled Agents regardless of the current node context

### Requirement: Shared Style Utilities for Common UI Patterns
The Web UI SHALL provide shared style utilities for frequently reused UI patterns to reduce duplication and keep visuals consistent.

Shared style utilities SHOULD cover:
- navigation chrome “glass” surfaces,
- settings-like list row hover/spacing,
- muted helper text,
- icon tiles used in list items.

#### Scenario: List row style is consistent across settings-like lists
- **WHEN** the UI renders settings-like lists (e.g. Settings overview, Notifications overview)
- **THEN** list rows share consistent spacing, hover behavior, and icon tile presentation via shared style utilities

### Requirement: Unused Legacy Views Are Removed
The Web UI SHALL remove unused legacy view files that are not referenced by the router/tests to reduce confusion and accidental regressions.

#### Scenario: No unused Settings legacy view remains
- **WHEN** the router is inspected
- **THEN** there is no unused legacy Settings view implementation file in the codebase

### Requirement: Route-Driven Document Titles
The Web UI SHALL set `document.title` based on the active route using localized titles.

#### Scenario: Jobs page updates the browser title
- **WHEN** the user navigates to `/jobs`
- **THEN** the browser tab title reflects the localized Jobs title

### Requirement: Theme Color Tracks Light/Dark Mode
The Web UI SHALL update `meta[name="theme-color"]` to a sensible value for the current theme so mobile browser chrome matches the UI.

#### Scenario: Dark mode updates theme-color
- **WHEN** the user switches to dark mode
- **THEN** `meta[name="theme-color"]` is set to a dark surface color

### Requirement: Focus Visibility and Reduced Motion
The Web UI SHALL provide a visible `:focus-visible` outline style for keyboard navigation and SHOULD respect `prefers-reduced-motion`.

#### Scenario: Keyboard focus is visible
- **WHEN** the user navigates UI controls with the keyboard
- **THEN** focused controls show a clear focus-visible indicator

### Requirement: Shared Empty-State Pattern
The Web UI SHALL provide a shared empty-state component/pattern and SHOULD use it on key list screens.

#### Scenario: Agents page empty state is clear
- **WHEN** there are no agents to show
- **THEN** the page shows a consistent empty-state presentation indicating there is no data

### Requirement: Latest Request Wins for Rapid Refresh Screens
For screens that can trigger multiple requests quickly (filters/pagination), the Web UI SHALL ensure the latest request wins and SHOULD cancel previous requests via `AbortController`.

#### Scenario: Notification queue filters do not produce stale results
- **WHEN** the user changes notification queue filters rapidly
- **THEN** the UI displays results from the latest filter selection only

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
- **THEN** the document `<html lang>` attribute becomes `en-US`

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

### Requirement: Settings Mobile TopBar Is Stable and Centered
On mobile viewports (`< md`), the Web UI SHALL render a dedicated TopBar for all pages under `/settings/**`.

The TopBar MUST:
- have a fixed height so page content does not shift vertically when navigating between Settings pages,
- reserve fixed left and right regions so the centered title does not shift horizontally,
- center the title text within the TopBar,
- reserve the right-side actions region, but it SHALL be empty in this version.

#### Scenario: Title does not shift when back button appears
- **WHEN** the user navigates from `/settings` to `/settings/storage`
- **THEN** the Settings mobile title remains horizontally centered
- **AND** the overall header height remains unchanged

### Requirement: TopBar Title and Back Target Are Route-Meta Driven
The Settings mobile TopBar SHALL derive its `title` and `back` behavior from route metadata for the current matched route.

The effective TopBar config SHALL be determined by the deepest matched route that provides TopBar metadata.

#### Scenario: Notifications subpage shows subpage title and returns to Notifications index
- **WHEN** the user opens `/settings/notifications/channels` on mobile
- **THEN** the TopBar title displays `Channels` (localized)
- **AND** tapping Back navigates to `/settings/notifications`

### Requirement: No Mobile Subtitle Under Settings Title
On mobile viewports, Settings pages SHALL NOT show the desktop-style subtitle under the Settings title.

Explanatory copy SHOULD be rendered as regular in-page gray helper text where needed.

#### Scenario: Settings subtitle hidden on mobile
- **WHEN** the user opens `/settings` on a mobile viewport
- **THEN** no subtitle is shown under the title area

### Requirement: Desktop Settings Sidebar Submenu
On desktop viewports (`>= md`), the Web UI SHALL present Settings sub-navigation as a sidebar submenu under a `Settings` parent item.

The `Settings` parent item SHALL only expand/collapse and MUST NOT navigate to a route when clicked.
The submenu SHALL include at least:
- Overview (`/settings`)
- Storage (`/settings/storage`)
- Notifications (`/settings/notifications`)

#### Scenario: Clicking Settings only expands
- **WHEN** the user clicks the `Settings` parent menu item on desktop
- **THEN** the Settings submenu expands/collapses
- **AND** the current route does not change

#### Scenario: Settings route highlights the correct submenu
- **WHEN** the current route is `/settings/notifications/queue`
- **THEN** the sidebar highlights the `Notifications` submenu item
- **AND** the Settings submenu is expanded

### Requirement: Settings Overview Page
The Web UI SHALL provide a Settings overview page at `/settings` that lists settings areas and navigates to their routes.

On mobile, the Settings overview SHOULD use an app-like list layout.

#### Scenario: Settings overview links to Storage
- **WHEN** the user taps `Storage` in Settings overview
- **THEN** the UI navigates to `/settings/storage`

### Requirement: Notifications Index Page and Mobile List-First Navigation
The Web UI SHALL provide a Notifications index page at `/settings/notifications` that lists Notifications subpages:
- Channels
- Destinations
- Templates
- Queue

On mobile viewports (`< md`), Notifications navigation MUST be list-first:
users enter the index list first, then navigate into a subpage.

On desktop, Notifications subpages MAY provide lightweight in-module navigation (e.g. tabs), but it MUST remain route-based so refresh/back/forward work correctly.

#### Scenario: Mobile enters Notifications via index list
- **WHEN** the viewport is `< md` and the user opens `/settings/notifications`
- **THEN** the UI displays a list of Notifications subpages

### Requirement: Settings Page Uses an Internal Submenu
The Web UI SHALL keep a single Settings entry in the main navigation and SHALL provide an internal Settings submenu to organize settings into multiple pages.

#### Scenario: Settings has subpages
- **WHEN** a user opens Settings
- **THEN** the UI presents a submenu to navigate to settings subpages

### Requirement: Notifications Settings UI (Channels/Destinations/Templates/Queue)
The Web UI SHALL provide a Notifications settings area with subpages for Channels, Destinations, Templates, and Queue.

On desktop viewports, the Notifications area MAY provide tabs for subpages, but it MUST remain route-based so refresh/back/forward work correctly.
On mobile viewports, the Notifications subpage navigation SHALL use a compact selector (e.g. segmented/select) and SHOULD avoid wide tables by using card lists.

#### Scenario: Desktop uses tabs but remains route-based
- **WHEN** a user switches from Destinations to Queue
- **THEN** the URL updates to the corresponding sub-route
- **AND** refreshing the browser keeps the user on that subpage

#### Scenario: Mobile navigation does not overflow
- **WHEN** the viewport is `< md`
- **THEN** notifications navigation is usable without horizontal overflow

### Requirement: Inline Validation Errors for Forms
For settings and notifications forms, the UI SHALL display errors inline in the form when possible (field-level), and SHALL still show a toast for unexpected failures.

#### Scenario: Invalid webhook URL shows field error
- **WHEN** a user saves a destination with an invalid webhook URL
- **THEN** the webhook URL form item shows an inline validation message

### Requirement: Forms Show Inline Error Reasons
For form-based workflows, the Web UI SHALL show inline error feedback that includes the failure reason, instead of only showing a generic toast message.

#### Scenario: Settings form shows invalid webhook error inline
- **WHEN** the user saves a WeCom bot secret with an invalid webhook URL
- **THEN** the UI shows an inline error message
- **AND** the error is associated with the webhook URL field when backend `details.field` is present

### Requirement: Known Error Codes Are Localized
The Web UI SHALL translate known backend `error` codes into localized (`zh-CN`/`en-US`) user-facing messages.

#### Scenario: Login invalid credentials is localized
- **WHEN** the backend responds with `error = "invalid_credentials"` during login
- **THEN** the UI displays a localized “invalid credentials” message in the current UI language

### Requirement: Fallback to Backend Message When Unknown
If the UI does not recognize a backend `error` code, it SHALL fall back to displaying the backend `message` (when provided) to avoid losing actionable information.

#### Scenario: Unknown backend error falls back to message
- **WHEN** the backend returns an unrecognized `error` code with a `message`
- **THEN** the UI displays that `message` inline for the user

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

### Requirement: Centralized Breakpoints and UI Constants
The Web UI SHALL centralize breakpoint definitions and other shared UI constants so responsive behavior and layout sizing remain consistent across the codebase.

#### Scenario: Breakpoint logic is centralized
- **WHEN** responsive behavior requires a breakpoint check
- **THEN** the code uses shared breakpoint constants rather than hard-coded values scattered across files

### Requirement: Safe Menu Navigation
Menu interactions in the Web UI SHALL NOT attempt to navigate to invalid or undefined routes.

#### Scenario: Menu key is invalid
- **WHEN** the menu emits an invalid/undefined key
- **THEN** the UI ignores it and no router navigation is attempted

### Requirement: Mobile Header Overflow Menu
On mobile viewports, the Web UI SHALL present global actions (language selection, theme toggle, logout) via a compact overflow menu so the header does not overflow.

#### Scenario: Mobile header actions do not overflow
- **WHEN** the viewport is `< md`
- **THEN** global actions are accessible without header content overflowing the screen

### Requirement: Mobile-Friendly Wizard Step Indicator
On mobile viewports, multi-step wizards in the Web UI SHALL use a compact step indicator (step x/total + progress bar) to avoid horizontal overflow, while desktop viewports may show the full stepper.

#### Scenario: Jobs wizard steps fit on mobile
- **WHEN** a user opens the Jobs create/edit wizard on a mobile viewport (`< md`)
- **THEN** the step indicator is readable without horizontal scrolling

### Requirement: Beta Label
The Web UI SHALL display a "Beta" label to indicate the UI is a test version.

#### Scenario: Beta tag is visible
- **WHEN** a user views the main navigation chrome (header/sidebar)
- **THEN** a "Beta" tag is displayed

### Requirement: Responsive Navigation and Layout
The Web UI SHALL be responsive and SHALL present exactly one navigation pattern per breakpoint:
- Mobile (`< md`): top bar with hamburger + drawer navigation.
- Desktop (`>= md`): persistent sidebar navigation (no drawer navigation).

#### Scenario: Desktop navigation does not include a drawer
- **WHEN** the viewport is `>= md`
- **THEN** the sidebar navigation is visible and the hamburger/drawer navigation is not shown

#### Scenario: Mobile navigation uses a drawer
- **WHEN** the viewport is `< md`
- **THEN** the sidebar navigation is not shown and navigation is accessible via the hamburger/drawer

### Requirement: Header and Content Alignment
On wide screens, the Web UI header controls SHALL align to the same container baseline as the main page content.

#### Scenario: Header aligns with content on wide screens
- **WHEN** the viewport is wide enough that a max-width container applies
- **THEN** header controls align horizontally with the main content container

### Requirement: Mobile Card Lists for Tabular Pages
For tabular list pages, the Web UI SHALL render a mobile-friendly card list on small screens while keeping tables on desktop.

#### Scenario: Jobs list renders as cards on mobile
- **WHEN** the user views the Jobs page on a mobile viewport (`< md`)
- **THEN** jobs are rendered as cards with primary actions available without horizontal scrolling

### Requirement: Dialog Sizing
Dialogs in the Web UI SHALL be constrained to sensible maximum widths on desktop and SHALL remain usable on mobile.

#### Scenario: Credential editor does not occupy full desktop width
- **WHEN** a user opens a credential editor dialog on a desktop viewport
- **THEN** the dialog width is constrained and does not span the full window width

### Requirement: Brand Mark Icon
The Web UI brand mark SHALL use Ionicons `ShieldCheckmark` (solid) and SHALL not appear visually distorted.

#### Scenario: Brand icon is not visually compressed
- **WHEN** the brand mark is displayed in the header or sidebar
- **THEN** the icon maintains its intended proportions and does not appear squeezed

### Requirement: UI Copy Punctuation Consistency
The Web UI SHALL render subtitles and short helper texts without trailing periods in both `zh-CN` and `en-US`.

#### Scenario: Login subtitle has no trailing punctuation
- **WHEN** the login page subtitle is rendered
- **THEN** it does not end with a trailing period character

### Requirement: Web UI for Jobs and Runs
The system SHALL provide a Web UI to create/edit jobs, trigger runs, and view run history and details.

#### Scenario: User triggers a run from the UI
- **WHEN** a user clicks "Run now" on a job
- **THEN** a new run is created and its status is visible in the UI

### Requirement: Web UI Internationalization (i18n)
The Web UI SHALL default to Simplified Chinese (`zh-CN`) and SHALL support switching between Simplified Chinese (`zh-CN`) and English (`en-US`), persisting the selection.

#### Scenario: Default language is Simplified Chinese
- **WHEN** a user opens the Web UI for the first time
- **THEN** the UI is displayed in `zh-CN`

#### Scenario: User switches language
- **WHEN** a user selects `en-US` from the language selector
- **THEN** the UI updates to English and persists the selection for future visits

### Requirement: Live Run Events
The Web UI SHALL display live run events/logs during execution.

#### Scenario: User watches live logs
- **WHEN** a run is executing
- **THEN** the UI receives and displays live events/log lines

### Requirement: Restore Wizard
The Web UI SHALL provide a restore wizard to select a restore point and restore destination and choose a conflict strategy.

#### Scenario: Restore to a new directory
- **WHEN** the user selects a restore point and destination directory
- **THEN** the system restores backup contents according to selected conflict strategy

### Requirement: Restore Drill Verification Wizard
The Web UI SHALL provide a verification wizard to run restore drills and view results.

#### Scenario: Run a restore drill
- **WHEN** the user starts a restore drill for a completed run
- **THEN** the system performs a full restore drill and reports pass/fail with details

### Requirement: Run Detail Events Provide Quick Navigation and Export
The Run Detail page SHALL provide quick navigation and export helpers for run events.

#### Scenario: User jumps to first error
- **GIVEN** the events list contains an error event
- **WHEN** the user invokes "jump to first error"
- **THEN** the UI scrolls the list to the first error event

#### Scenario: User exports filtered events
- **GIVEN** the user has applied filters/search to the events list
- **WHEN** the user exports events
- **THEN** the UI exports the filtered events as JSON

### Requirement: Run Detail Summary/Progress Remains Accessible on Desktop
The Run Detail page SHALL keep the Summary + Progress panel accessible while the user browses the Details area on desktop.

#### Scenario: Summary/Progress remains visible while browsing details
- **GIVEN** the user scrolls within the Details area on a desktop viewport
- **THEN** the Summary + Progress panel remains visible without requiring the user to scroll back to the top

### Requirement: Run Detail Presents a Cohesive Header and Action Area
The Run Detail page SHALL present run status, target information, and primary actions as a cohesive header/action area.

#### Scenario: Header uses localized labels and consistent actions
- **GIVEN** a run is loaded
- **THEN** the run status is displayed using localized text
- **AND** target information is displayed using product-friendly labels
- **AND** Restore/Verify actions are disabled when the run is not eligible

### Requirement: Run Detail Consolidates Secondary Sections
The Run Detail page SHALL consolidate Events, Operations, and Summary into a single Details area.

#### Scenario: Desktop avoids long scrolling with tabs
- **GIVEN** the user is on a desktop viewport
- **THEN** the page presents Events / Operations / Summary as tabs
- **AND** empty sections do not render large placeholder tables/cards

#### Scenario: Mobile presents the same Details tabs
- **GIVEN** the user is on a mobile viewport
- **THEN** the page presents the same Events / Operations / Summary tabs in a mobile-friendly layout

### Requirement: Run Summary Hides Empty Blocks
The Run Detail page SHALL avoid rendering empty summary blocks.

#### Scenario: Summary only renders detail blocks when present
- **GIVEN** a run has a summary payload
- **WHEN** optional summary fields are absent
- **THEN** the page does not render empty placeholder panels for those fields

### Requirement: Run Detail Uses Responsive “Overview + Progress” Layout
The Run Detail page SHALL present the overview and progress information in a responsive layout that is readable on both desktop and mobile.

#### Scenario: Desktop uses a two-column first screen
- **GIVEN** the user is on a desktop viewport
- **THEN** the Run Detail page shows “Overview” and “Progress” side-by-side

#### Scenario: Mobile uses a single-column first screen
- **GIVEN** the user is on a mobile viewport
- **THEN** the Run Detail page stacks “Overview” above “Progress”

### Requirement: Run Detail Events Are Scan-Friendly
The Run Detail page SHALL present run events in a scan-friendly list.

#### Scenario: Events are shown as a timeline list with details
- **GIVEN** a run has events
- **THEN** the page shows events in a list optimized for scanning (timestamp + level + message)
- **AND** users can open event details to view any structured fields

### Requirement: Run Summary Shows Highlights and Raw JSON
The Run Detail page SHALL show a readable summary with an option to view the raw JSON.

#### Scenario: Summary shows structured highlights with a raw JSON fallback
- **GIVEN** a run has a summary payload
- **THEN** the page shows key summary highlights in a readable format
- **AND** the raw JSON is available via a collapsible section with a copy affordance

### Requirement: Job Editor Can Select Artifact Format
The Web UI SHALL allow selecting an artifact format for a job:
- `archive_v1` (default)
- `raw_tree_v1`

#### Scenario: Raw-tree disables encryption controls
- **WHEN** the user selects artifact format `raw_tree_v1`
- **THEN** encryption controls are disabled or hidden
- **AND** the UI explains that raw-tree does not support encryption

### Requirement: Offline-Executed Runs Remain Understandable
When runs are executed on an Agent while the Hub is unreachable and later synced, the Web UI SHALL keep the user experience understandable and consistent.

The UI MAY annotate runs as “executed offline” and/or show delayed ingestion time.

#### Scenario: User can distinguish delayed ingestion
- **WHEN** an offline-executed run is synced later to the Hub
- **THEN** the UI can indicate that the run executed while offline (optional) without breaking run viewing workflows

### Requirement: Foundation Tokens for Visual Consistency
The Web UI SHALL define a small set of globally stable “foundation” tokens (e.g., radii and motion) that are not theme-specific and are used consistently across shared components.

#### Scenario: Foundation tokens control shared primitives
- **GIVEN** the UI renders common primitives (cards, toolbars, panels)
- **WHEN** the foundation tokens are updated
- **THEN** shared primitives update consistently without page-by-page overrides

### Requirement: Muted Text Uses Theme Tokens (Not Opacity)
The Web UI SHALL render secondary/muted text using theme tokens (e.g. `--app-text-muted`) and SHOULD avoid using opacity utilities to communicate semantic “muted” meaning for text.

#### Scenario: Muted text remains consistent across themes
- **WHEN** the user switches between light/dark mode or theme presets
- **THEN** muted text remains readable and visually consistent
- **AND** it does not become too faint or too strong due to stacked opacity

### Requirement: Dividers and Borders Use Theme Tokens
The Web UI SHALL render dividers and subtle borders using theme tokens (e.g. `--app-border`) rather than hard-coded black/white translucency utilities.

#### Scenario: List separators match the active theme
- **GIVEN** the UI renders a list with separators
- **WHEN** the user switches themes
- **THEN** separators and subtle borders continue to match the active surface hierarchy

### Requirement: Standardized Component Recipes
The Web UI SHALL provide and use standardized recipes for common UI patterns, including at minimum:
- Card / inset panel,
- list row,
- list/filter toolbars,
- tags/badges (status vs neutral),
- data tables,
- mono/code blocks and keycap hints.

#### Scenario: Two pages share the same look for the same pattern
- **GIVEN** two pages render the same pattern (e.g. a clickable list row)
- **WHEN** both pages are viewed in light and dark mode
- **THEN** the pattern looks consistent (spacing, radius, divider, hover/pressed, typography)

### Requirement: Guardrails Prevent Non-Token Styling Regressions
The repository SHALL include an automated guardrail check that detects reintroduced non-token styling patterns in `ui/src` (e.g. hard-coded Tailwind “semantic colors” and disallowed arbitrary values) and fails CI when violations are introduced.

#### Scenario: CI blocks a non-token color regression
- **WHEN** a contributor introduces a forbidden non-token styling pattern in `ui/src`
- **THEN** the guardrail check fails
- **AND** the failure message guides the contributor toward the token-based alternative

### Requirement: Developer-Facing UI Style Guide (EN + zh-CN)
The project SHALL document the Web UI visual system in the docs site, including:
- token inventory and meaning,
- approved spacing/typography/radius scales,
- recipes for common patterns,
- and do/don’t examples for consistency.

#### Scenario: Contributor can learn the rules without tribal knowledge
- **GIVEN** a new contributor needs to add or modify a Web UI screen
- **WHEN** they open the developer docs
- **THEN** they can find the UI style guide and apply the documented recipes

### Requirement: Visual Accessibility Remains Intact
The consistency refactor SHALL preserve focus-visible indicators, SHOULD maintain adequate contrast for text (including muted text) in light/dark mode, and SHOULD respect reduced motion preferences.

#### Scenario: Keyboard focus remains clearly visible
- **WHEN** the user navigates the UI using the keyboard
- **THEN** focus-visible styling remains clearly visible and consistent

### Requirement: Web UI SHALL render envelope diagnostics in maintenance and snapshot management views
Maintenance and snapshot management diagnostic surfaces SHALL prioritize canonical envelope diagnostics when available.

#### Scenario: Task detail includes envelope diagnostics
- **GIVEN** task-related diagnostics include an event envelope
- **WHEN** user opens maintenance or snapshot task details
- **THEN** UI SHALL display envelope-based localized message/hint and key protocol details
- **AND** UI SHALL expose retriable and context metadata where available

### Requirement: Web UI SHALL preserve fallback compatibility for legacy task errors
UI SHALL continue to render meaningful diagnostics when canonical envelope fields are missing.

#### Scenario: Only legacy task error fields are available
- **GIVEN** task details provide `last_error_kind/last_error` without envelope
- **WHEN** user inspects the task diagnostics
- **THEN** UI SHALL render legacy diagnostics without regression
- **AND** generic localized fallback SHALL be shown if both envelope and legacy diagnostics are unavailable

### Requirement: Shared Modal Shell SHALL Enforce Container-vs-Body Layout Boundaries
The web UI SHALL enforce a shared modal layout contract where viewport-bounding size constraints are applied to the modal container layer, while body scrolling and internal flow are handled by the modal body layer.

#### Scenario: Long-form dialog remains viewport-bounded
- **GIVEN** a dialog with long form content (for example, task create/edit)
- **WHEN** the dialog opens on desktop or mobile
- **THEN** the modal container height remains bounded by viewport-safe limits
- **AND** the user scrolls dialog content inside the modal body instead of the page growing beyond intended modal bounds

#### Scenario: Plain body mode does not bypass overflow safety
- **GIVEN** a dialog using `scrollBody=false`
- **WHEN** content height exceeds available body space
- **THEN** body layout still respects bounded height and overflow rules defined by the shared modal contract
- **AND** footer actions remain reachable without layout breakage

### Requirement: Modal Layout Regressions SHALL Be Covered by Unit Tests
The web UI SHALL include unit tests for shared modal layout contract behavior so height/overflow regressions are detected before merge.

#### Scenario: Contract tests cover container and body responsibilities
- **GIVEN** the shared modal shell and a representative long-form dialog
- **WHEN** unit tests run in CI
- **THEN** tests verify container-level sizing inputs and body-level scrolling behavior are applied to the expected layers
- **AND** regressions that move viewport constraints into incorrect layers fail tests

### Requirement: Reusable Dialog Components SHALL Reuse Shared Modal Shell
Reusable Jobs and Run dialog components SHALL render through the shared modal shell to keep modal body/footers and optional slot behavior consistent.

#### Scenario: Component-level dialogs use the shared shell without behavior regressions
- **GIVEN** the user opens reusable dialogs such as job editor/deploy/runs/restore/verify and run-event details/events
- **WHEN** dialog content, actions, and optional header slots are rendered
- **THEN** the dialog components use the shared modal shell wrapper
- **AND** existing submit/cancel flows, emitted events, and scroll behavior remain unchanged

### Requirement: Run List Dialogs SHALL Ignore Stale Responses
Run list dialogs SHALL only apply data from the latest open/load request for the current job context.

#### Scenario: User switches job quickly while run list is loading
- **GIVEN** the user opens run list for job A and immediately opens run list for job B
- **WHEN** job A's request resolves after job B
- **THEN** the dialog shows runs for job B only
- **AND** stale responses from earlier requests are ignored

### Requirement: Run Event Streams SHALL Reuse Shared Lifecycle Control
Run event consumers SHALL share the same stream lifecycle logic for connect, reconnect backoff, and sequence deduplication.

#### Scenario: WebSocket reconnect and message dedupe remains consistent
- **GIVEN** multiple run event consumers in the UI
- **WHEN** connections drop and later recover
- **THEN** both consumers use the same reconnect backoff policy
- **AND** duplicate or old event sequences are not appended

### Requirement: Picker Session Opening SHALL Stage Transition and Initial Refresh
The web UI SHALL stage picker session startup so modal visibility/enter transition and first list refresh are sequenced to reduce frame contention during dialog open.

#### Scenario: Open transition is not blocked by immediate heavy refresh work
- **GIVEN** a user opens a directory picker dialog
- **WHEN** the session starts
- **THEN** the dialog is shown before heavy list refresh work begins
- **AND** initial data refresh still occurs automatically within the same open session

### Requirement: Picker Table Height Measurement SHALL Avoid Redundant Open-Frame Work
The web UI SHALL measure picker table body height with a stable lifecycle that avoids unnecessary repeated open-frame measurements.

#### Scenario: Open lifecycle performs stable measurement setup
- **GIVEN** a picker dialog opens
- **WHEN** table-body max height is initialized
- **THEN** the measurement lifecycle performs only the required initial measurement/setup steps
- **AND** subsequent re-measurements are driven by meaningful size changes instead of redundant chained frame callbacks

### Requirement: Picker Open/Refresh Performance Guards SHALL Include Unit Tests
The web UI SHALL include unit tests that assert picker open sequencing and measurement lifecycle behavior to prevent regressions.

#### Scenario: Unit tests detect sequencing regressions
- **GIVEN** picker session and table-height composables
- **WHEN** unit tests run in CI
- **THEN** tests verify open sequencing, refresh trigger timing, and measurement lifecycle expectations
- **AND** regressions that reintroduce open-time contention fail tests

### Requirement: Agents Management Dialogs SHALL Reuse Shared Modal Shell
Agents management dialogs SHALL reuse the shared modal shell component for consistent body spacing, footer actions, and scroll containment.

#### Scenario: Agents labels and bulk dialogs follow shared modal structure
- **GIVEN** the user opens labels, bulk sync, or bulk labels dialogs on the Agents page
- **WHEN** dialog content is rendered and actions are shown in the footer
- **THEN** the dialogs use the shared modal shell wrapper
- **AND** existing form behavior and submit/cancel semantics remain unchanged

### Requirement: Core Page Dialogs SHALL Reuse Shared Modal Shell
Core page dialogs SHALL reuse the shared modal shell so body spacing, footer actions, and scroll containment stay consistent across Jobs/Settings/Snapshots surfaces.

#### Scenario: Page-level dialogs render through shared shell
- **GIVEN** the user opens dialogs on Job Snapshots, Job Workspace, Bulk Operations, Settings Storage, Notifications Destinations, or Maintenance Cleanup pages
- **WHEN** dialog content and footer actions are rendered
- **THEN** the dialogs use the shared modal shell wrapper
- **AND** existing titles, header-extra actions, and submit/cancel behavior remain unchanged

### Requirement: Large Web UI Screens SHALL Separate Orchestration From Presentation
Large Web UI screens/components SHALL extract shared orchestration logic (query sync, async loading, bulk actions, or picker state) into dedicated composables/modules.

#### Scenario: Refactored view keeps existing behavior
- **GIVEN** a large screen with filters, pagination, and action handlers
- **WHEN** orchestration logic is extracted to composables/modules
- **THEN** visible behavior and route/query semantics stay unchanged
- **AND** the screen becomes easier to maintain with smaller focused units

### Requirement: Jobs Modal Flows SHALL Have Direct Regression Coverage
Critical jobs modal flows SHALL include direct component tests for open/submit/error behavior.

#### Scenario: Modal flow changes trigger test failures when behavior regresses
- **GIVEN** jobs modals such as editor/deploy/runs/restore/verify
- **WHEN** open flow, primary action, or API error handling regresses
- **THEN** component-level tests fail and surface the regression

### Requirement: Node-Scoped Route Paths SHALL Use Shared Builders
Node-scoped navigation paths SHALL be constructed and parsed through shared route helpers to prevent drift in encoding and suffix handling.

#### Scenario: Node id and suffix normalization stay consistent
- **GIVEN** a node id that may contain special characters
- **WHEN** UI code builds jobs/settings paths or switches node context
- **THEN** node route helpers are used for encoding and suffix normalization
- **AND** resulting paths remain consistent across pages

### Requirement: Clipboard Feedback SHALL Reuse Shared Behavior
Views that expose copy actions SHALL use a shared copy+feedback primitive for success/error toasts.

#### Scenario: Copy action feedback is consistent across views
- **GIVEN** the user clicks copy actions on agents/settings pages
- **WHEN** clipboard write succeeds or fails
- **THEN** the same success/error feedback behavior is used

### Requirement: Icon-Only Actions SHALL Provide Accessible Labels
Icon-only action controls SHALL provide explicit accessible labels through a shared component contract.

#### Scenario: Help buttons remain readable by assistive technologies
- **GIVEN** an icon-only help button in desktop or mobile layouts
- **WHEN** the control is rendered
- **THEN** it exposes a non-empty accessible label

### Requirement: Picker Modal Wrappers SHALL Align With Shared Modal Shell
Picker modal wrappers and picker confirmation card dialogs SHALL align with shared modal shell structure while preserving existing flows.

#### Scenario: Picker confirm modal keeps behavior after shell alignment
- **GIVEN** the user opens picker current-directory confirmation
- **WHEN** modal shell wrappers are aligned
- **THEN** title/content/footer actions remain unchanged
- **AND** body/footer spacing and structure follow shared shell conventions

### Requirement: Picker Lists SHALL Ignore Stale Responses
Picker list surfaces SHALL protect refresh/load-more data state from stale asynchronous responses.

#### Scenario: Older request resolves after a newer filter/path request
- **GIVEN** a picker list has triggered request A and then request B with newer state
- **WHEN** request A resolves after request B
- **THEN** request A result SHALL NOT override the currently displayed rows/cursor state
- **AND** loading indicators SHALL settle according to the latest active request

### Requirement: Picker Loaded-Row Selection Semantics SHALL Be Shared
Picker surfaces that support multi-row selection SHALL share one loaded-row selection model for select-all, invert, and shift-range behaviors.

#### Scenario: Shared loaded-row selection is reused across picker modals
- **GIVEN** two picker modals expose loaded-row selection controls
- **WHEN** the user triggers select-all/invert/shift-range actions
- **THEN** both modals SHALL derive the next selected set from the same shared selection logic
- **AND** selection from non-loaded pages/paths SHALL remain preserved where applicable

### Requirement: Jobs Workspace Filters SHALL Use Shared Filter Modeling
Jobs workspace list filtering SHALL use the shared list filter model for active-count/chip generation and clear behavior.

#### Scenario: Jobs filter chips/count use shared model
- **GIVEN** the user applies search and/or select-based filters in Jobs workspace
- **WHEN** filter chips and active filter count are rendered
- **THEN** chips/count SHALL be derived through the shared filter model utility
- **AND** clear-all SHALL reset to Jobs-defined defaults without page-local duplicated chip/count logic

### Requirement: Picker/List Query Serialization SHALL Be Shared
Picker/list request query parameter serialization for common filters SHALL be provided through shared helpers.

#### Scenario: Shared serialization keeps query semantics stable
- **GIVEN** picker-like list requests with search, kind, dotfiles, type sort, size range, and sort options
- **WHEN** requests are serialized to query parameters
- **THEN** shared helpers SHALL produce consistent parameter keys and value normalization
- **AND** migrated surfaces SHALL preserve existing backend contract semantics

### Requirement: Debounce And Abort Guards SHALL Be Shared Utilities
List views with debounced refresh and abort-aware error handling SHALL reuse shared utility helpers.

#### Scenario: Debounced refresh + abort guard reuse
- **GIVEN** list views that debounce refresh or swallow abort cancellation errors
- **WHEN** those views are implemented
- **THEN** debounce scheduling and abort-error detection SHALL be provided by shared utility helpers
- **AND** equivalent views SHALL avoid reimplementing ad-hoc timer/abort detection logic

### Requirement: Core List Pages Reuse Shared Route-Filter Hydration
Core list pages SHALL reuse shared route-query parsing helpers for hydrating filter state from URL query values.

#### Scenario: Route query values are parsed with shared helper semantics
- **GIVEN** a list page reads filter values from `route.query`
- **WHEN** query values are strings, arrays, comma-separated strings, or invalid values
- **THEN** page filter state is hydrated via shared parsing helpers
- **AND** unknown values are ignored without breaking defaults

### Requirement: Server-Paginated List Pages Reuse Shared Pagination Behavior
Server-paginated list pages SHALL reuse shared pagination component behavior and shared page-size options to avoid per-page interaction drift.

#### Scenario: List pages expose consistent pagination controls
- **GIVEN** two or more server-paginated list pages
- **WHEN** users change page or page size
- **THEN** each page uses the same pagination component interaction model
- **AND** page-size option defaults are sourced from shared constants

### Requirement: Picker Open/Reset Lifecycle Uses Shared Model
Picker modals with large open/reset state blocks SHALL reuse a shared picker lifecycle/reset model.

#### Scenario: Picker opens with clean deterministic state
- **GIVEN** a picker modal is opened repeatedly with different contexts
- **WHEN** the picker initializes state for a new session
- **THEN** reset logic is executed through shared lifecycle helpers
- **AND** stale local UI state from prior sessions is cleared consistently

### Requirement: Per-Item Busy State Uses Shared Infrastructure
Views that track request-in-flight status by entity id SHALL reuse shared busy-state composables.

#### Scenario: Busy flags are managed through shared id helpers
- **GIVEN** a list view supports row-level operations with loading states
- **WHEN** an operation starts or finishes for a specific row id
- **THEN** busy state is updated via shared id-based helpers
- **AND** local pages avoid duplicating map-clone/delete boilerplate

### Requirement: Store List Query Serialization Reuses Shared Builders
Store list APIs SHALL reuse shared query serialization helpers for common list/filter/pagination parameter patterns.

#### Scenario: List stores serialize common parameters consistently
- **GIVEN** stores build `URLSearchParams` for paginated/filterable list endpoints
- **WHEN** filters and pagination parameters are applied
- **THEN** shared serializer helpers construct common query keys and values
- **AND** existing endpoint-specific parameter names remain unchanged

### Requirement: Jobs Workspace Row Rendering Is Componentized
Jobs workspace SHALL split duplicated row/table rendering into shared subcomponents while preserving current actions and selection behavior.

#### Scenario: Desktop/mobile row actions remain behaviorally equivalent after extraction
- **GIVEN** Jobs workspace renders list/table rows across desktop and mobile layouts
- **WHEN** users select rows, open details, run now, or open overflow actions
- **THEN** extracted row-rendering components preserve the same action semantics
- **AND** duplicated per-layout row markup is reduced through reusable components

### Requirement: Jobs Results Summary Uses Explicit Visible And Filtered Semantics
The Jobs workspace SHALL display list metrics that clearly distinguish currently visible items from filtered total matches.

#### Scenario: Jobs list shows visible count and filtered total
- **GIVEN** the Jobs workspace list is loaded
- **WHEN** the current page has M rows and the filtered dataset has N total rows
- **THEN** the UI shows both M and N in the results summary
- **AND** the summary no longer renders an ambiguous duplicated total expression

### Requirement: Jobs Mobile Filter Visibility Matches Desktop Parity
The Jobs workspace SHALL surface active filter conditions in mobile list mode using the same filter-chip source of truth as desktop layouts.

#### Scenario: Mobile layout shows and clears active filters via chips
- **GIVEN** the user applies search or filter controls in Jobs mobile list mode
- **WHEN** the list content renders
- **THEN** active filter chips are visible above list content
- **AND** chip close / clear actions reset the same underlying filter model used in desktop layouts

### Requirement: Jobs Row Activation Semantics Are Explicit And Accessible
Jobs list rows SHALL separate row-main activation from nested action controls using explicit interactive elements with keyboard-friendly semantics.

#### Scenario: Row-main activation and row-action activation do not conflict
- **GIVEN** a Jobs list row with row-main activation and nested action controls
- **WHEN** the user triggers a row action control
- **THEN** only the action executes
- **AND** row-main navigation/select behavior does not fire

#### Scenario: Row-main activation supports keyboard interaction
- **GIVEN** a Jobs list row in list mode
- **WHEN** keyboard users focus and activate the row-main trigger
- **THEN** the same row-main behavior executes as pointer activation

### Requirement: Notifications Queue Provides Explicit Empty-State Variants
Notifications Queue SHALL provide explicit empty-state guidance for loading-empty, base empty, and filtered-no-results states.

#### Scenario: Queue shows context-aware empty state messaging
- **GIVEN** Notifications Queue has no rows to display
- **WHEN** state is loading-empty, no queued entries, or filter-no-match
- **THEN** the page shows state-appropriate empty-state title/description/actions
- **AND** users can recover quickly (for example, via clear filters or refresh)

### Requirement: List Pagination Summaries Include Visible Range And Total
List pages that use shared pagination SHALL expose a consistent visible-range summary with total count.

#### Scenario: Jobs, Agents, and Notifications show range summary consistently
- **GIVEN** a paginated list page among Jobs, Agents, or Notifications Queue
- **WHEN** pagination footer is rendered
- **THEN** the summary includes visible start/end indices and total count
- **AND** formatting is consistent across these pages

### Requirement: Agents Mobile Cards Prioritize Primary Information
Agents mobile cards SHALL prioritize primary scan fields and move secondary metadata into progressive disclosure.

#### Scenario: Secondary metadata is collapsed by default on mobile cards
- **GIVEN** the user views Agents on mobile viewport
- **WHEN** agent cards render
- **THEN** primary identity/status/actions remain directly visible
- **AND** less-critical metadata is accessible through an explicit expand/collapse affordance

### Requirement: Key Async Row Actions Provide In-Flight Feedback
List row actions for high-frequency operations SHALL provide immediate in-flight feedback and duplicate-submit prevention.

#### Scenario: Triggering row action sets local busy state
- **GIVEN** a user triggers a key row action (for example, "Run now" or notification retry/cancel)
- **WHEN** the async request is in flight
- **THEN** the corresponding control shows loading/busy feedback
- **AND** repeated triggers for the same row action are temporarily disabled until completion

### Requirement: Search-Driven List Refresh Uses A Shared Debounce Cadence
Search-driven list refresh behavior SHALL use a shared debounce cadence across list pages that support text query search.

#### Scenario: Jobs and Agents search refresh cadence is consistent
- **GIVEN** the user types in Jobs or Agents search input
- **WHEN** query text changes rapidly
- **THEN** refresh requests are deferred by the same debounce interval before fetch
- **AND** the effective cadence is consistent across both pages

### Requirement: Page Surfaces SHALL Expose Consistent Visual Hierarchy
The web UI SHALL provide consistent hierarchy levels for page titles, section titles, and metadata text across shared layout components.

#### Scenario: Shared headers and list shells render consistent hierarchy
- **GIVEN** pages render `PageHeader` and list scaffolds
- **WHEN** the user navigates between Jobs and Agents list pages
- **THEN** title, subtitle, and metadata text use consistent visual hierarchy classes
- **AND** hierarchy levels do not depend on per-page ad-hoc utility combinations

### Requirement: Shared Surfaces SHALL Reduce Decorative Noise
The web UI SHALL tune shared chrome tokens so backgrounds, borders, and shadows prioritize data readability over decoration.

#### Scenario: Content remains dominant across themes
- **GIVEN** the user views a list-heavy page in any supported theme
- **WHEN** cards, toolbars, and list containers are displayed
- **THEN** surface styling uses reduced-noise defaults (subtler background intensity and chrome)
- **AND** primary content contrast remains clear in light and dark modes

### Requirement: List Scaffolds SHALL Use Unified Spacing Rhythm
Shared list layout components SHALL enforce consistent vertical spacing for toolbar, content, and pagination zones.

#### Scenario: Two list pages share spacing cadence
- **GIVEN** two pages use `ListPageScaffold`
- **WHEN** both render toolbar, content area, and pagination
- **THEN** the vertical spacing cadence is consistent
- **AND** page-local overrides are no longer required for basic rhythm

### Requirement: Dense List Metadata SHALL Follow Shared Emphasis Rules
List rows and data-table secondary metadata SHALL use shared low-emphasis styles to preserve scanability in dense datasets.

#### Scenario: Row metadata remains readable without competing with primary labels
- **GIVEN** a list row contains primary text and secondary metadata
- **WHEN** the row is rendered in list or table mode
- **THEN** primary text remains visually dominant
- **AND** secondary metadata uses shared reduced-emphasis styles

### Requirement: List Filters Use Shared Modeling For Active State And Chips
List pages that expose filter controls SHALL use a shared local filter-model utility for active-state derivation, clear behavior, and active-filter chip generation.

#### Scenario: Shared filter model drives clear and chip state
- **GIVEN** a list page with search and/or select-based filters
- **WHEN** filters are changed
- **THEN** active-filter count/chips are derived from the shared filter model
- **AND** clear-all resets the same model back to page-defined defaults

### Requirement: List Filter Controls Reuse Shared Field Wrappers
List pages that render select-based toolbar filters SHALL reuse shared list filter field wrappers for consistent sizing and toolbar presentation.

#### Scenario: Filter select controls render with consistent width and layout
- **GIVEN** list pages with one or more select filters in toolbar regions
- **WHEN** those filters are rendered on desktop and mobile breakpoints
- **THEN** shared field wrappers provide consistent container sizing and select control layout
- **AND** pages avoid repeating ad-hoc width utility blocks for equivalent filter fields

### Requirement: Active Filter Visibility Is Consistent Across Core List Pages
Agents, Notifications Queue, Maintenance Cleanup, and Job Snapshots SHALL display active-filter chips with a consistent clear affordance.

#### Scenario: Migrated list pages show active chips and support clear-all
- **GIVEN** the user applies one or more filters on Agents, Notifications Queue, Maintenance Cleanup, or Job Snapshots
- **WHEN** list content is displayed
- **THEN** an active-filter chip row appears using the shared component path
- **AND** users can clear all filters from the same row-level clear action

### Requirement: Picker Modals Reuse Shared Filter Modeling For Active Chips
Run entries and path picker modals SHALL reuse the shared local filter model for active-filter count/chips and clear behavior.

#### Scenario: Picker modals keep filter-count/chip behavior via shared model
- **GIVEN** the user applies filters in RunEntries picker or PathPicker modal
- **WHEN** the filters toolbar and active-chip row render
- **THEN** active filter count/chips are derived through shared filter-model utilities
- **AND** clear-all and per-chip close behaviors remain functional without page-local duplicated chip/count logic

### Requirement: List Filters SHALL Use Shared Field Presentation
List pages SHALL present filter labels and controls through shared field presentation primitives in both inline and stacked modes.

#### Scenario: Inline and stacked filter panels share visual structure
- **GIVEN** a page renders filters inline on desktop and stacked on constrained layouts
- **WHEN** the user opens or interacts with filters
- **THEN** filter label/control structure remains consistent between modes
- **AND** pages avoid duplicating handcrafted label/select/switch wrappers

### Requirement: List Pages SHALL Expose Unified Filter Summary Feedback
List pages SHALL expose result summary and active-filter chips through a shared summary row pattern.

#### Scenario: Filter effects are visible at a predictable location
- **GIVEN** one or more filters are active
- **WHEN** the list content renders
- **THEN** result count and active filter chips are shown in a consistent summary row
- **AND** clearing filters uses a shared interaction pattern

### Requirement: List State Feedback SHALL Reuse Shared Presenter
Loading, base-empty, and filtered-empty states SHALL be rendered through a shared state presenter component.

#### Scenario: Empty-state semantics stay consistent across pages
- **GIVEN** Jobs and Agents list pages can be loading, empty, or filtered-empty
- **WHEN** there are no rows to render
- **THEN** the page uses a shared state presenter
- **AND** action affordances (create/clear) are surfaced with consistent structure

### Requirement: Interactive Icons SHALL Use Shared Size and Tone Semantics
The UI SHALL provide a shared icon wrapper for interactive controls so icon size and semantic tone remain consistent.

#### Scenario: Action icons keep consistent dimensions
- **GIVEN** multiple action controls render icons across list pages
- **WHEN** the controls are displayed together
- **THEN** icons use shared size semantics instead of arbitrary per-page sizing
- **AND** semantic tone mapping is applied consistently

### Requirement: Modal Shells SHALL Reuse a Shared Structural Wrapper
The UI SHALL render critical card-style modals through a shared modal shell wrapper with consistent spacing and scroll behavior.

#### Scenario: Modal content and footer remain stable across pages
- **GIVEN** the user opens modal dialogs on Jobs and Agents pages
- **WHEN** dialogs include long content or loading transitions
- **THEN** body spacing and footer alignment remain consistent
- **AND** modal content area scroll behavior is predictable

### Requirement: Shared Micro-Interaction Rules SHALL Be Applied to Core List Surfaces
Core list surfaces SHALL use shared motion rules for hover/focus/press feedback with reduced-motion fallback.

#### Scenario: Users receive consistent interaction feedback
- **GIVEN** users interact with list rows, cards, and filter trigger controls
- **WHEN** hover/focus/press states occur
- **THEN** interactions use shared duration/easing tokens
- **AND** reduced-motion preference disables non-essential transitions

### Requirement: Global Shell SHALL Emphasize Primary Content And Unified Actions
The web UI SHALL present navigation chrome and global actions so that primary page content remains visually dominant and desktop/mobile global actions are grouped consistently.

#### Scenario: Desktop shell keeps content primary while grouping global actions
- **GIVEN** the user is on any authenticated desktop page
- **WHEN** the shell renders navigation and topbar controls
- **THEN** the content area remains the dominant visual surface
- **AND** global actions are grouped together instead of competing with page-level actions

#### Scenario: Mobile shell exposes navigation and global actions in one coherent system
- **GIVEN** the user is on any authenticated mobile page
- **WHEN** they open global navigation
- **THEN** they can access navigation, node context, and global actions without hunting across separate menus

### Requirement: Dashboard SHALL Prioritize Actionable First-Screen Information
The dashboard SHALL place high-priority health signals and recent activity ahead of lower-priority analytical context.

#### Scenario: Users see urgent status before secondary analytics
- **GIVEN** the dashboard contains health, run, and trend information
- **WHEN** the page loads on the first screen
- **THEN** actionable status cards and recent activity appear before trend-only context
- **AND** the user can navigate directly to the relevant management surface from those areas

### Requirement: Jobs Workspace SHALL Reduce Redundant Top-Level Layout Decisions
The Jobs workspace SHALL minimize redundant top-level layout choices so users can enter the primary workflow without first interpreting multiple overlapping display modes.

#### Scenario: Users enter the jobs workflow through a clear primary mode
- **GIVEN** the user opens the Jobs workspace
- **WHEN** the top-level toolbar is shown
- **THEN** the page presents a clear primary workflow mode
- **AND** secondary display choices do not require interpreting multiple overlapping groups of toggles

### Requirement: Empty And Auth States SHALL Provide Guided Next Steps
The web UI SHALL turn first-run empty states and the login screen into guided surfaces with clear next actions and product context.

#### Scenario: Agents empty state guides first-time setup
- **GIVEN** there are no connected agents
- **WHEN** the Agents page renders
- **THEN** the empty state explains the setup flow in concise steps
- **AND** the primary setup action is visually obvious

#### Scenario: Login page communicates context and help affordances
- **GIVEN** the user is on the login page
- **WHEN** the page renders
- **THEN** the UI communicates what Bastion manages and offers supporting guidance beyond the credential form

### Requirement: Authenticated Pages SHALL Expose Main Landmark Semantics
Authenticated pages SHALL expose a clear main content landmark to improve navigation for assistive technologies.

#### Scenario: Main content region is discoverable to assistive technology
- **GIVEN** the user navigates an authenticated page with a screen reader or accessibility audit
- **WHEN** the page structure is evaluated
- **THEN** the primary content area is exposed as a main landmark

