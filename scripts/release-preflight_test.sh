#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

bash "${root_dir}/scripts/release-preflight.sh" --tag v0.1.0 --output "${tmp_dir}/release-notes.md"
grep -q '^## \[v0.1.0\] - 2026-01-13$' "${tmp_dir}/release-notes.md"

if bash "${root_dir}/scripts/release-preflight.sh" --tag 0.1.0 --output "${tmp_dir}/bad-tag.md"; then
  echo "expected invalid tag format check to fail" >&2
  exit 1
fi

if bash "${root_dir}/scripts/release-preflight.sh" --tag v9.9.9 --output "${tmp_dir}/missing-tag.md"; then
  echo "expected missing changelog section check to fail" >&2
  exit 1
fi

echo "release preflight tests passed"
