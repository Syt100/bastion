#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/release-preflight.sh --tag vX.Y.Z [--output release-notes.md] [--run-ci] [--check-tag-availability]

Options:
  --tag vX.Y.Z              Release tag to validate and extract.
  --output <path>           Output path for extracted release notes. Default: release-notes.md
  --run-ci                  Run full CI script after changelog checks.
  --check-tag-availability  Fail if the target tag already exists locally or on origin.
  -h, --help                Show this help.
USAGE
}

die() {
  echo "error: $*" >&2
  exit 1
}

check_tag_availability() {
  local tag="$1"
  local remote_output

  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    die "tag already exists locally: ${tag}"
  fi

  if git remote get-url origin >/dev/null 2>&1; then
    if ! remote_output="$(git ls-remote --tags origin "refs/tags/${tag}" 2>/dev/null)"; then
      die "failed to query tags from origin (rerun without --check-tag-availability if offline)"
    fi
    if [ -n "${remote_output}" ]; then
      die "tag already exists on origin: ${tag}"
    fi
  else
    echo "warn: remote 'origin' not configured; skipping remote tag check" >&2
  fi
}

main() {
  local tag=""
  local output="release-notes.md"
  local run_ci=0
  local check_tag=0

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --tag)
        [ "$#" -ge 2 ] || die "--tag requires a value"
        tag="$2"
        shift 2
        ;;
      --output)
        [ "$#" -ge 2 ] || die "--output requires a value"
        output="$2"
        shift 2
        ;;
      --run-ci)
        run_ci=1
        shift
        ;;
      --check-tag-availability)
        check_tag=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "unknown argument: $1"
        ;;
    esac
  done

  [ -n "${tag}" ] || die "--tag is required"
  if [[ ! "${tag}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    die "tag must look like vX.Y.Z (received: ${tag})"
  fi

  if [ "${check_tag}" -eq 1 ]; then
    echo "==> Tag availability: ${tag}"
    check_tag_availability "${tag}"
  fi

  echo "==> Changelog: check"
  bash scripts/changelog.sh check

  echo "==> Changelog: tests"
  bash scripts/changelog_test.sh

  echo "==> Changelog: extract (${tag})"
  bash scripts/changelog.sh extract --tag "${tag}" --output "${output}"

  if ! grep -Eq '^- ' "${output}"; then
    die "extracted release notes contain no bullet entries: ${output}"
  fi

  if [ "${run_ci}" -eq 1 ]; then
    echo "==> CI: scripts/ci.sh"
    bash scripts/ci.sh
  fi

  cat <<EOF
[OK] Release preflight passed for ${tag}
Release notes: ${output}
Next steps:
  git tag ${tag}
  git push origin main --tags
EOF
}

main "$@"
