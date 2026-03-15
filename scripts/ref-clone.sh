#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <git-url> [name]" >&2
}

if (( $# < 1 || $# > 2 )); then
  usage
  exit 64
fi

url="$1"
name="${2:-${url##*/}}"
name="${name%.git}"
target=".ref/${name}"

mkdir -p .ref

if [[ -d "${target}/.git" ]]; then
  git -C "$target" fetch --all --tags --prune
  current_branch="$(git -C "$target" branch --show-current)"
  if [[ -n "$current_branch" ]]; then
    git -C "$target" pull --ff-only
  fi
else
  if [[ -e "$target" ]]; then
    echo "Target '$target' exists and is not a git checkout." >&2
    exit 1
  fi
  git clone "$url" "$target"
fi

echo "Reference checkout is available at $target"
