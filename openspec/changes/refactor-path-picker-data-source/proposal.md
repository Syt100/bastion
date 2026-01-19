# Change: Refactor filesystem path picker into a data-source driven path picker

## Why
`FsPathPickerModal` currently hardcodes filesystem listing via `/api/nodes/{node}/fs/list` and bakes UI behaviors (filters, sorting, columns, errors) into one component.

This makes it expensive to add new "browsers" for other backends (e.g. WebDAV, S3) and increases the risk of inconsistencies when multiple pickers evolve.

## What Changes
- Introduce a generic, data-source driven path picker that:
  - Consumes a `PickerDataSource` interface (list/navigate + error mapping)
  - Uses a capability declaration to enable/disable UI features per data source (filters, sorting, columns, pagination, selection modes)
- Keep `FsPathPickerModal` as a thin wrapper (or alias) that wires the filesystem data source and preserves the existing external API/UX.
- Ensure the architecture can support future data sources (WebDAV/S3) without rewriting the picker UI.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - New generic picker + data-source types under `ui/src/components/pickers/`

## Non-Goals
- Implementing a WebDAV or S3 browser in this change.
- Changing backend APIs.
- Intentionally changing picker UX/behavior (refactor first; new features follow in separate changes).
