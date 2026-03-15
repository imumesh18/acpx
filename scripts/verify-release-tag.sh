#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <tag>" >&2
}

if (( $# != 1 )); then
  usage
  exit 64
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to verify release tags." >&2
  exit 1
fi

tag="${1#refs/tags/}"
version="${tag#v}"
crate_version="$(
  cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[] | select(.name == "acpx") | .version'
)"

if [[ "$version" != "$crate_version" ]]; then
  echo "Tag (${tag}) does not match Cargo.toml version (${crate_version})." >&2
  exit 1
fi

if ! grep -Eq "^## \\[${crate_version//./\\.}\\]" CHANGELOG.md; then
  echo "CHANGELOG.md does not contain an entry for ${crate_version}." >&2
  exit 1
fi
