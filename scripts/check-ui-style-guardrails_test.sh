#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

mkdir -p "${tmp_dir}/ok" "${tmp_dir}/bad-color" "${tmp_dir}/bad-font"

cat > "${tmp_dir}/ok/Example.vue" <<'EOF'
<template>
  <div class="text-[var(--app-text-muted)] bg-[var(--app-surface-2)] border-[var(--app-border)] text-sm">
    ok
  </div>
</template>
EOF

cat > "${tmp_dir}/bad-color/Example.vue" <<'EOF'
<template>
  <div class="text-red-500">bad</div>
</template>
EOF

cat > "${tmp_dir}/bad-font/Example.vue" <<'EOF'
<template>
  <div class="text-[17px]">bad</div>
</template>
EOF

bash "${root_dir}/scripts/check-ui-style-guardrails.sh" "${tmp_dir}/ok"

if bash "${root_dir}/scripts/check-ui-style-guardrails.sh" "${tmp_dir}/bad-color"; then
  echo "expected bad-color guardrail to fail" >&2
  exit 1
fi

if bash "${root_dir}/scripts/check-ui-style-guardrails.sh" "${tmp_dir}/bad-font"; then
  echo "expected bad-font guardrail to fail" >&2
  exit 1
fi

echo "ui style guardrail tests passed"
