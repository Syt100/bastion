# Design: Source snapshots + direct upload

## Part A: Source snapshots

### Interfaces
Introduce a small abstraction layer:
```
trait SnapshotProvider {
  fn probe(root: &Path) -> ProbeResult; // supported? why not?
  fn create_snapshot(root: &Path, run_id: &str) -> SnapshotHandle;
}

struct SnapshotHandle {
  provider: String,
  snapshot_path: PathBuf,  // read-only view
  cleanup: fn() -> Result<(), Error>, // best-effort
}
```

### Modes
- `off`: do not attempt snapshots.
- `auto`: attempt a snapshot; if unavailable, emit a warning event and continue with best-effort consistency detection.
- `required`: if snapshot cannot be created, fail the run with `error_code="snapshot_unavailable"`.

### Providers (phased)
Phase 1 (deliver first):
- Linux Btrfs (read-only subvolume snapshot)

Phase 2+ (scaffolding + later):
- ZFS snapshots
- Windows VSS
- macOS/APFS (optional, complex)

### Btrfs implementation sketch
- Preconditions:
  - Source root is on a Btrfs filesystem and is a subvolume (or is within a subvolume that can be snapshotted at the dataset boundary).
- Snapshot location:
  - Under the run directory: `run_dir/source_snapshot/<name>` (so it is tied to the run lifecycle).
- Commands:
  - Create: `btrfs subvolume snapshot -r <root> <snapshot>`
  - Delete: `btrfs subvolume delete <snapshot>`
- Permissions:
  - Requires sufficient privileges. This MUST run on the agent host (not in the browser) and must be explicitly enabled/allowed by configuration.

### Run events + summary
Run events:
- `snapshot_started`: includes provider + root
- `snapshot_ready`: includes provider + snapshot_path
- `snapshot_unavailable`: includes reason

Run summary:
- `summary.filesystem.snapshot`: `{ mode, provider?, status, reason? }`

UI renders snapshot status in run detail and job editor.

### Security constraints
- No shell string interpolation: execute commands with args.
- Enforce allowlist: only snapshot within configured roots.
- Ensure cleanup runs even on failure (best-effort background cleanup is acceptable if run ends unexpectedly).

## Part B: Direct upload / reduced staging

### Current state
- `archive_v1` already supports rolling part upload (minimizes local staging to approximately one part).
- `raw_tree_v1` stages an entire `staging/data` directory before copying/uploading, which is disk-heavy.

### LocalDir raw_tree: “direct data path” mode (Phase 1)
Goal: avoid duplicating the data tree under local staging for local_dir targets.

Approach:
- Before building the run, create a `data` directory inside the target run dir:
  - `<base_dir>/<job_id>/<run_id>/data`
- Create `stage_dir/data` as a symlink (or junction on Windows) pointing to that target `data` directory.
- Build raw_tree as usual; it will write into `stage_dir/data/...` which resolves directly to the target.
- During the upload stage, the local_dir target copy step becomes a no-op (files already exist by size); we may optionally short-circuit to avoid the extra traversal.
- The completion marker is still written last, preserving atomicity.

Pros:
- Low refactor risk; leverages existing raw_tree builder and target store logic.
- Eliminates the biggest staging duplication for local targets.

Cons:
- Still traverses the tree during “upload” unless optimized.
- Needs careful handling on Windows (junction creation) and symlink permissions.

### WebDAV raw_tree: direct upload (Phase 2, optional)
Goal: avoid staging full tree for WebDAV targets too.

Approach (higher complexity):
- Introduce a new “raw_tree uploader” pipeline that:
  - walks the source tree
  - streams each file directly to WebDAV (PUT) while hashing
  - writes entries index concurrently from the same bytes uploaded (single-read)
  - writes manifest + entries index + complete at the end
- Keep atomicity:
  - `complete` is written last; restore/verify only considers runs with `complete`
- Failure cleanup:
  - best-effort delete of partially uploaded files, or leave partial without `complete`.

### Testing strategy
- Provider unit tests: probe/create/cleanup (mock command runner).
- Integration tests:
  - required snapshot mode fails when provider unavailable.
  - local_dir raw_tree direct mode writes data to target before complete marker exists.
  - complete marker is last; incomplete runs are ignored by list/restore logic.

