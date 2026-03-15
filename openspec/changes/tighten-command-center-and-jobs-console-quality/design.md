## Context

The first wave of shell and Jobs work fixed the major architectural issues: stable routes, top-level information architecture, a Command Center landing page, a three-pane Jobs workspace, and a full-page editor. The remaining issues are no longer about "which page owns the workflow?" They are about execution quality:

- professional control-plane copy versus slogan-like marketing phrasing
- dominant task surfaces versus visually even metadata blocks
- object-first detail framing versus collection controls lingering as primary chrome
- mobile task completion versus mechanically compressed desktop patterns

This change is intentionally a quality-tightening pass. It does not redefine product scope or data contracts. It hardens how the existing surfaces should feel and behave.

## Goals / Non-Goals

**Goals:**
- codify operator-facing copy standards for command-center and jobs surfaces
- ensure actionable states outrank neutral metadata visually
- make dedicated job detail flows read as job detail, not as list pages with extra content
- make mobile job authoring usable without horizontally compressed multi-step chrome
- improve touch-first discoverability of the most common actions

**Non-Goals:**
- introducing new primary surfaces beyond the existing shell plan
- redefining Jobs or Command Center data models
- replacing current read models with new backend endpoints
- restyling unrelated settings pages in this change

## Decisions

### 1. Control-console copy will be operational, concise, and evidence-first

Primary operational surfaces must avoid promotional or slogan-shaped language. Headings, subtitles, helper text, and empty states should sound like a control plane used by operators who need to assess status and decide what to do next.

Rules:

- prefer direct statements over framing slogans
- prefer evidence and consequence over abstract reassurance
- avoid marketing-style lines such as "put everything in one panel" when the page can state what it actually shows
- empty and healthy states should still explain why the state is quiet, not just that counts are zero

Examples:

- preferred: `查看失败运行、恢复准备度和待处理事项。`
- avoid: `把备份风险、关键活动和恢复信心放到同一个操作面板里。`

### 2. Command Center keeps the first screen compact and action-led

The top of Command Center should communicate:

- overall state
- why that state matters
- what the operator should open next

Neutral metadata such as scope echo, generation time, and non-urgent totals remain useful, but they should be visibly subordinate. A healthy system still shows evidence, not only a blank set of zeros.

Implications:

- the hero area should stay compact enough that at least one actionable section is visible on common desktop heights
- readiness cautions or attention lists should visually outrank summary counters
- timestamps and range/scope echoes should read as support metadata, not peer-level status blocks

### 3. Dedicated job detail routes are object-first, not collection-first

Desktop Jobs may retain list context, but the dedicated detail route should clearly privilege the selected job. Collection controls such as saved-view management, list-mode toggles, and full filter toolbars should not dominate the object page once a job has been opened directly.

Implications:

- the selected job header should be the first clear semantic anchor
- return context should be represented as a back action, scope chip, or saved-view chip rather than as a full collection toolbar
- object actions such as `Run now`, `Edit`, `Open recent run`, and `Refresh` should live close to the job header
- supporting panes should summarize recent runs, warnings, and readiness without making the page feel like a dashboard collage

### 4. Mobile job authoring uses compressed progress, not a full horizontal step strip

The desktop stepper can show all stages simultaneously, but mobile cannot rely on seven horizontally compressed step labels. The authoring flow should foreground the current step, overall progress, and a secondary way to inspect or jump between steps.

Implications:

- mobile should show current-step identity plus progress such as `步骤 1/7`
- step jumping may live in a sheet, menu, or expandable progress control
- the current step must remain readable without horizontal scrolling or crushed labels
- validation should anchor near the active field/section and near the main action bar

### 5. Mobile summaries collapse before primary inputs do

Configuration summary and risk summary remain important on mobile, but they must not displace the active form. On narrow screens they should collapse into drawers, accordions, sticky peek cards, or equivalent subordinate containers.

Implications:

- the first viewport should prioritize the current authoring fields and primary actions
- summary blocks should be reachable without forcing the operator to scroll through full secondary panels after every step transition
- risk signals that block progress should still surface near the primary action area even when the full summary is collapsed

### 6. Touch workflows need explicit primary action affordances

Common touch workflows should not depend on icon-only discovery where the action is central to the task.

Implications:

- tapping a job row/card should clearly open the job, independent from explicit quick actions
- mobile detail pages should expose primary actions with text labels, not icon-only affordances as the only path
- list and detail actions should meet comfortable touch sizing and spacing expectations

## Risks / Trade-offs

- [Tightening copy may require touching many i18n strings] -> keep wording concise and centralize replacements where existing string keys can be reused
- [Reducing top-of-page chrome may remove some context operators rely on] -> preserve return context and scope context as compact chips/actions instead of removing them entirely
- [Collapsing summaries on mobile could hide useful state] -> keep blocking or high-severity warnings mirrored near the primary action area

## Migration Plan

1. Add the quality-tightening spec deltas for Command Center, Jobs, the job editor, and the design system.
2. Apply the updated rules as ongoing UI work continues on adjacent operational surfaces.
3. Use the resulting patterns when implementing Runs, Fleet, Integrations, and System so new pages do not inherit the older weaker tone and hierarchy.

Rollback:
- these changes tighten presentation rules only; rollback is simply to revert the affected UI and copy adjustments if they prove too aggressive

## Open Questions

- Should professional-copy rules later be generalized into a broader product-writing guide beyond operational surfaces?
- Should the mobile editor summary be a bottom sheet, an inline accordion, or a sticky peek card by default?
