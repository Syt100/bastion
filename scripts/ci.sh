#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root_dir}"

echo "==> Secrets: gitleaks"
gitleaks_version="v8.30.0"
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks_bin="gitleaks"
else
  if ! command -v go >/dev/null 2>&1; then
    echo "gitleaks not found and go is not installed. Install gitleaks: https://github.com/gitleaks/gitleaks" >&2
    exit 1
  fi
  tools_dir="${HOME}/.cache/bastion-tools/bin"
  mkdir -p "${tools_dir}"
  GOBIN="${tools_dir}" go install "github.com/zricethezav/gitleaks/v8@${gitleaks_version}"
  gitleaks_bin="${tools_dir}/gitleaks"
fi

"${gitleaks_bin}" detect --source "${root_dir}" --redact --no-banner --exit-code 1

echo "==> UI: install"
npm ci --prefix ui

echo "==> UI: build"
npm run build-only --prefix ui

echo "==> Rust: fmt"
cargo fmt --check

echo "==> Rust: clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> Rust: test"
cargo test --workspace

echo "==> UI: test"
npm test --prefix ui
