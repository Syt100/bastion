# Design: WebDAV safety limits, explicit direct upload config, and alert digest

## 1. Goals
- Prevent WebDAV targets from being overwhelmed by raw-tree upload traffic (especially many small files).
- Make WebDAV raw-tree direct upload an explicit, user-controlled job/pipeline setting (no surprise enablement).
- De-noise warnings in the runs list: show only high-signal summaries; run detail keeps full evidence.

## 2. Explicit direct upload config (job/pipeline)

### Proposed job spec shape
Extend `PipelineV1` with WebDAV upload settings (conceptual JSON):

```json
{
  "pipeline": {
    "format": "raw_tree_v1",
    "encryption": { "type": "none" },
    "webdav": {
      "raw_tree_direct": {
        "mode": "off|auto|on",
        "resume_by_size": true,
        "limits": {
          "concurrency": 4,
          "put_qps": 20,
          "head_qps": 50,
          "mkcol_qps": 50,
          "burst": 10
        }
      }
    }
  }
}
```

Notes:
- Default `mode=off` to keep behavior explicit and safe.
- `auto` means: allow the engine to enable direct upload when the pipeline/target supports it.
- `on` means: require direct upload; invalid combos MUST be rejected at validation time.

### Validation rules
- If `raw_tree_direct.mode != off`:
  - `pipeline.format MUST be raw_tree_v1`
  - `target.type MUST be webdav`
- If `source.consistency_policy == fail` AND `source.upload_on_consistency_failure != true`:
  - `raw_tree_direct.mode MUST be off`
  - Rationale: direct upload writes data to the target before policy is evaluated; allowing it would violate “fail means do not upload”.

### Engine enablement logic
- Remove the current “auto-enable when conditions match” behavior.
- Engine decides direct upload solely based on job spec:
  - `off` → no direct upload
  - `auto` → enable only when supported, else fall back to staged upload (and emit an info/warn event)
  - `on` → must enable; otherwise fail the run early with a clear error

## 3. WebDAV safety limits (concurrency + rate limiting)

### Where limits live
Implement request limiting inside `bastion_targets::WebdavClient` so it applies uniformly across:
- staged upload (`bastion_targets::webdav::store_run` + raw-tree data dir traversal)
- direct upload (`bastion_backup` raw-tree WebDAV sink)

### Limiter model
We need **two** mechanisms:
- **Concurrency limit**: bound simultaneous in-flight requests.
- **Rate limit**: smooth request bursts over time (and optionally allow small bursts).

Implementation sketch:
- `Semaphore` for concurrency.
- A simple leaky-bucket gate for each method group (PUT/HEAD/MKCOL):
  - Store `next_allowed_at: Instant` in a `Mutex`.
  - For each request, compute a delay based on `1/qps` and sleep if needed.
  - Optional `burst` permits short bursts without sleeping (implemented as “allow up to N immediate permits per window”).

### Backpressure
If the server returns HTTP 429/503 and a `Retry-After` header:
- Sleep for the indicated duration (bounded).
- Retry (up to `max_attempts`), still respecting the limiter.

### Preserving atomic semantics
The `complete.json` marker MUST still be written last:
- staged upload: ensure all data + manifest/index are uploaded before complete.
- direct upload: ensure packaging completes (all direct payload PUTs done) before `store_run()` uploads complete.

## 4. Making uploads concurrent (without breaking ordering)

### Staged raw-tree upload to WebDAV
Current raw-tree staged upload traverses `data/` and uploads each file serially (HEAD then PUT).
Update to:
- Traverse directory, enqueue upload tasks, and process with bounded concurrency (e.g. `buffer_unordered`).
- Per-file logic remains resumable-by-size.
- Limiter still applies as a safety valve.

### Direct raw-tree upload to WebDAV during packaging
Current direct upload is synchronous (scan → per file `block_on(PUT)`), which keeps memory bounded and preserves deterministic index ordering.

We keep it synchronous for now, but enforce WebDAV request limits via `WebdavClient` so direct upload cannot overwhelm the server:
- rate limiting (PUT/HEAD/MKCOL qps + burst)
- concurrency cap (max in-flight requests)

A future optimization could introduce a bounded concurrent pipeline, but it is not required for safety.

## 5. Alert digest / de-noising strategy

### Problem
Displaying every warning signal in the runs list is noisy. The runs list should show:
- high-signal, actionable summaries
- bounded badges (max N) in a stable priority order

### Signals to expose
For filesystem runs:
- `issues_errors_total`, `issues_warnings_total` (from summary filesystem issues)
- consistency breakdown:
  - `consistency_total`
  - `consistency_signal_total = replaced_total + deleted_total + read_error_total`

### Runs list rendering rules (UI)
Order (max 3 badges):
1) `Errors: X` (if `issues_errors_total > 0`)
2) `Warnings: Y` (if `issues_warnings_total > 0`)
3) Consistency:
   - If `consistency_signal_total > 0` → show `Source risk: {consistency_signal_total}`
   - Else if `consistency_total >= THRESHOLD` → show `Source changed: {consistency_total}`
   - Else → hide (de-noise low-volume changed-only)

Run detail continues to show full breakdown + samples + evidence.

### Control-plane API shape
Extend `GET /api/jobs/:id/runs` items with the fields above, including for running runs:
- If summary exists, derive from summary (authoritative).
- Else, derive from latest events (best-effort early signal), for both issues and consistency.

## 6. Testing
- WebDAV client limiter:
  - unit tests for concurrency ceiling (server records peak in-flight)
  - unit tests for rate smoothing (time-window assertions)
- Staged raw-tree upload:
  - integration test with many small files asserts bounded concurrency and completion semantics
- Job spec validation:
  - invalid combos rejected (`mode != off` but target/format mismatch)
  - consistency fail + upload disabled rejects direct upload
- Runs list digest:
  - API tests for summary precedence and running-run fallback
- UI:
  - runs list shows only high-signal badges in priority order
  - detail page continues to show full consistency samples/evidence
