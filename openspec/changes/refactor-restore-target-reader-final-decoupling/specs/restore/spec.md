## ADDED Requirements

### Requirement: Restore Runtime MUST Consume Driver Reader Contract
Restore and run-entry index APIs MUST consume run artifacts through `TargetRunReader` contract
methods rather than target-kind-specific reader variants.

#### Scenario: Restore opens reader through registry without target-kind switch
- **WHEN** restore runtime resolves a successful run
- **THEN** it opens a target reader from registry
- **AND** complete marker checks, manifest/index reads, and payload/raw-tree reads use reader contract methods

#### Scenario: Run-entry index fetch uses reader contract download semantics
- **WHEN** run-entry listing needs `entries.index.zst`
- **THEN** runtime uses reader `head_size` + `get_to_file` contract for remote readers
- **AND** local-path hints may be used as an optimization without introducing target-kind branches

### Requirement: Artifact Stream MUST Reuse Driver Reader Contract
Hub artifact stream APIs MUST use the same target reader contract as restore path and only keep
node-local fast path branching.

#### Scenario: Artifact stream generic path is target-agnostic
- **WHEN** client opens stream for a successful run on a non-local target
- **THEN** stream uses reader contract operations for manifest/complete/index/payload reads
- **AND** adding a new target reader does not require artifact-stream target-kind branches
