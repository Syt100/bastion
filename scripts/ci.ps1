Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$rootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $rootDir

Write-Host "==> Secrets: gitleaks"
$gitleaks = Get-Command gitleaks -ErrorAction SilentlyContinue
if (-not $gitleaks) {
  $go = Get-Command go -ErrorAction SilentlyContinue
  if (-not $go) {
    throw "gitleaks not found and Go is not installed. Install gitleaks: https://github.com/gitleaks/gitleaks"
  }

  $gitleaksVersion = "v8.30.0"
  $baseDir = if ($env:LOCALAPPDATA) { $env:LOCALAPPDATA } else { $env:USERPROFILE }
  $toolsDir = Join-Path $baseDir "bastion-tools\\bin"
  New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

  $env:GOBIN = $toolsDir
  go install "github.com/zricethezav/gitleaks/v8@$gitleaksVersion"
  $env:PATH = "$toolsDir;$env:PATH"
}

gitleaks detect --source $rootDir --redact --no-banner --exit-code 1

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

Write-Host "==> Docs: install"
npm ci --prefix docs

Write-Host "==> Docs: generate reference"
cargo run -p bastion --bin docgen -- --check

Write-Host "==> Docs: build"
$env:DOCS_BASE = "/docs/"
npm run build --prefix docs
