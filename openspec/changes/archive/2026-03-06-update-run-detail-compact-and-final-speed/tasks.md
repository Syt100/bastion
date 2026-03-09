## 1. Spec
- [x] 1.1 Add web-ui spec delta for compact run detail and final speed display
- [x] 1.2 Run `openspec validate update-run-detail-compact-and-final-speed --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Run Detail layout: reduce whitespace; avoid card stretching
- [x] 2.2 Progress panel: compute/display final speed on completion when missing
- [x] 2.3 Visual polish: tighten typography/alignment in Overview/Progress

## 3. Tests
- [x] 3.1 Update/add unit tests for the new final-speed computation logic

## 4. Validation
- [x] 4.1 Run `npm test --prefix ui`
- [x] 4.2 Run `npm --prefix ui run build`
- [x] 4.3 Run `cargo test --workspace`

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message with Modules/Tests)
