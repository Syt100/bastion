#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

cat > "${tmp_dir}/ok.md" <<'EOF'
# Changelog

## [Unreleased]

### Added
- Unreleased feature note.

### Changed
- Unreleased behavior note.

## [v1.2.3] - 2026-02-12

### Fixed
- First fix.

## [v1.2.2] - 2026-02-01

### Security
- Patch details.
EOF

cat > "${tmp_dir}/bad-category.md" <<'EOF'
# Changelog

## [Unreleased]

### Misc
- Invalid category.

## [v1.2.3] - 2026-02-12

### Fixed
- First fix.
EOF

cat > "${tmp_dir}/bad-version-placeholder.md" <<'EOF'
# Changelog

## [Unreleased]

### Added
- Upcoming item.

## [v1.2.3] - 2026-02-12

### Security
- _No user-facing changes yet._
EOF

bash "${root_dir}/scripts/changelog.sh" check --file "${tmp_dir}/ok.md"

bash "${root_dir}/scripts/changelog.sh" extract --file "${tmp_dir}/ok.md" --tag v1.2.3 --output "${tmp_dir}/release-notes.md"
grep -q '^## \[v1.2.3\] - 2026-02-12$' "${tmp_dir}/release-notes.md"
grep -q '^- First fix\.$' "${tmp_dir}/release-notes.md"

if bash "${root_dir}/scripts/changelog.sh" check --file "${tmp_dir}/bad-category.md"; then
  echo "expected bad-category check to fail" >&2
  exit 1
fi

if bash "${root_dir}/scripts/changelog.sh" extract --file "${tmp_dir}/ok.md" --tag v9.9.9 --output "${tmp_dir}/missing.md"; then
  echo "expected extract with missing tag to fail" >&2
  exit 1
fi

if bash "${root_dir}/scripts/changelog.sh" check --file "${tmp_dir}/bad-version-placeholder.md"; then
  echo "expected bad-version-placeholder check to fail" >&2
  exit 1
fi

echo "changelog script tests passed"
