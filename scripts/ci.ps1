Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$rootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $rootDir

Write-Host "==> Rust: fmt"
cargo fmt --check

Write-Host "==> Rust: clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

Write-Host "==> Rust: test"
cargo test --workspace

Write-Host "==> UI: install"
npm ci --prefix ui

Write-Host "==> UI: test"
npm test --prefix ui
