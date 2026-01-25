## 1. Spec
- [x] 1.1 Add web-ui spec delta for compact run detail and final speed display
- [x] 1.2 Run `openspec validate update-run-detail-compact-and-final-speed --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Run Detail layout: reduce whitespace; avoid card stretching
- [ ] 2.2 Progress panel: compute/display final speed on completion when missing
- [ ] 2.3 Visual polish: tighten typography/alignment in Overview/Progress

## 3. Tests
- [ ] 3.1 Update/add unit tests for the new final-speed computation logic

## 4. Validation
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm --prefix ui run build`
- [ ] 4.3 Run `cargo test --workspace`

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
