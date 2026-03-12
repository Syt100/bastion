#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_dir="${1:-${root_dir}/ui/src}"

if [ ! -d "${target_dir}" ]; then
  echo "target directory does not exist: ${target_dir}" >&2
  exit 1
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) is required for UI style guardrails" >&2
  exit 1
fi

checks=(
  "Hard-coded text palette colors::(dark:)?text-(red|amber|orange|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone)-[0-9]{2,3}"
  "Theme-incompatible white/black chrome colors::(dark:)?(bg-(white|black)|border-(white|black)|divide-(white|black))/[0-9]{1,3}"
  "Arbitrary font sizes::text-\\[[0-9]+px\\]"
)

failed=0
for rule in "${checks[@]}"; do
  label="${rule%%::*}"
  pattern="${rule##*::}"
  if rg -n --glob '*.{vue,ts,tsx,css}' "${pattern}" "${target_dir}"; then
    echo "UI style guardrail failed: ${label}" >&2
    failed=1
  fi
done

if [ "${failed}" -ne 0 ]; then
  exit 1
fi

echo "UI style guardrails passed"
