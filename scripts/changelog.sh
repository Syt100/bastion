#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/changelog.sh check [--file CHANGELOG.md]
  bash scripts/changelog.sh extract --tag vX.Y.Z [--file CHANGELOG.md] [--output release-notes.md]
USAGE
}

die() {
  echo "error: $*" >&2
  exit 1
}

check_changelog() {
  local file="$1"

  [ -f "$file" ] || die "changelog file not found: $file"

  if ! grep -Eq '^## \[Unreleased\]$' "$file"; then
    die "missing required section header: ## [Unreleased]"
  fi

  if ! grep -Eq '^## \[(v?[0-9]+\.[0-9]+\.[0-9]+[^]]*)\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$file"; then
    die "missing version section header like: ## [v0.1.0] - YYYY-MM-DD"
  fi

  local unreleased_line first_version_line
  unreleased_line="$(awk '$0=="## [Unreleased]" { print NR; exit }' "$file")"
  first_version_line="$(awk '/^## \[(v?[0-9]+\.[0-9]+\.[0-9]+[^]]*)\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$/ { print NR; exit }' "$file")"

  if [ -z "$unreleased_line" ] || [ -z "$first_version_line" ] || [ "$unreleased_line" -ge "$first_version_line" ]; then
    die "## [Unreleased] must appear before the first version section"
  fi

  if ! awk '
    /^### / {
      name = substr($0, 5)
      if (!(name == "Added" || name == "Changed" || name == "Deprecated" || name == "Removed" || name == "Fixed" || name == "Security")) {
        printf("%d:%s\n", NR, $0)
        invalid = 1
      }
    }
    END { exit invalid ? 1 : 0 }
  ' "$file"; then
    die "unsupported category heading found (allowed: Added, Changed, Deprecated, Removed, Fixed, Security)"
  fi

  if ! awk '
    $0 == "## [Unreleased]" { in_unreleased = 1; next }
    /^## / && in_unreleased { exit }
    in_unreleased && /^### / { has_category = 1 }
    END { exit has_category ? 0 : 1 }
  ' "$file"; then
    die "## [Unreleased] must include at least one ### category heading"
  fi

  if ! awk '
    BEGIN {
      in_release = 0
      invalid = 0
    }
    /^## \[/ {
      in_release = ($0 ~ /^## \[(v?[0-9]+\.[0-9]+\.[0-9]+[^]]*)\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$/)
      next
    }
    in_release && $0 == "- _No user-facing changes yet._" {
      printf("%d:%s\n", NR, $0)
      invalid = 1
    }
    END { exit invalid ? 1 : 0 }
  ' "$file"; then
    die "released version sections must not include placeholder lines like: - _No user-facing changes yet._"
  fi
}

extract_release_notes() {
  local file="$1"
  local tag="$2"
  local output="$3"
  local plain_tag

  [ -f "$file" ] || die "changelog file not found: $file"

  if [[ ! "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    die "tag must look like vX.Y.Z (received: $tag)"
  fi

  plain_tag="${tag#v}"

  if awk -v tag="$tag" -v plain_tag="$plain_tag" '
    BEGIN {
      capture = 0
      found = 0
    }
    {
      if (match($0, /^## \[([^]]+)\]/, m)) {
        version = m[1]
        if (capture) {
          exit
        }
        if (version == tag || version == plain_tag || ("v" version) == tag) {
          capture = 1
          found = 1
        }
      }

      if (capture) {
        print $0
      }
    }
    END {
      if (!found) {
        exit 64
      }
    }
  ' "$file" > "$output"; then
    :
  else
    local code=$?
    if [ "$code" -eq 64 ]; then
      die "no changelog section found for tag: $tag"
    fi
    die "failed to extract changelog section for tag: $tag"
  fi

  if ! grep -q '^## \[' "$output"; then
    die "extracted release notes are empty for tag: $tag"
  fi
}

main() {
  local cmd="${1:-}"
  local file="CHANGELOG.md"

  if [ -z "$cmd" ]; then
    usage
    exit 1
  fi
  shift || true

  case "$cmd" in
    check)
      while [ "$#" -gt 0 ]; do
        case "$1" in
          --file)
            [ "$#" -ge 2 ] || die "--file requires a value"
            file="$2"
            shift 2
            ;;
          -h|--help)
            usage
            exit 0
            ;;
          *)
            die "unknown argument for check: $1"
            ;;
        esac
      done

      check_changelog "$file"
      ;;

    extract)
      local tag=""
      local output="release-notes.md"

      while [ "$#" -gt 0 ]; do
        case "$1" in
          --file)
            [ "$#" -ge 2 ] || die "--file requires a value"
            file="$2"
            shift 2
            ;;
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
          -h|--help)
            usage
            exit 0
            ;;
          *)
            die "unknown argument for extract: $1"
            ;;
        esac
      done

      [ -n "$tag" ] || die "--tag is required for extract"
      extract_release_notes "$file" "$tag" "$output"
      ;;

    -h|--help)
      usage
      ;;

    *)
      die "unknown command: $cmd"
      ;;
  esac
}

main "$@"
