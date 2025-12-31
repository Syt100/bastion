#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root_dir}"

echo "==> Rust: fmt"
cargo fmt --check

echo "==> Rust: clippy"
cargo clippy --workspace --all-targets --all-features

echo "==> Rust: test"
cargo test --workspace

echo "==> UI: install"
npm ci --prefix ui

echo "==> UI: test"
npm test --prefix ui

