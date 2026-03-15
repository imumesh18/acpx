#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 [version]" >&2
}

if (( $# > 1 )); then
  usage
  exit 64
fi

if ! command -v git-cliff >/dev/null 2>&1; then
  echo "git-cliff is required to prepare a release." >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to prepare a release." >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Release preparation requires a clean worktree." >&2
  exit 1
fi

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(git cliff --config cliff.toml --bumped-version 2>/dev/null | sed 's/^v//')"
fi

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "Version '$version' is not a supported SemVer release string." >&2
  exit 1
fi

tag="v${version}"

if git rev-parse --verify --quiet "$tag" >/dev/null; then
  echo "Tag '$tag' already exists." >&2
  exit 1
fi

current_version="$(
  cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[] | select(.name == "acpx") | .version'
)"

if [[ "$current_version" != "$version" ]]; then
  VERSION="$version" perl -0pi -e 's/(\[package\][^\[]*?\nversion = ")[^"]+(")/$1$ENV{VERSION}$2/s' Cargo.toml
  cargo check --all-features >/dev/null
fi

git cliff --config cliff.toml --tag "$tag" --output CHANGELOG.md
./scripts/quality-gates.sh

git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): ${tag}"
git tag -a "$tag" -m "$tag"

echo "Prepared ${tag}. Push it with: git push origin HEAD --follow-tags"
