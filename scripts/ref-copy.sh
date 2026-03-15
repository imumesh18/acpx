#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <source> <name>" >&2
}

if (( $# != 2 )); then
  usage
  exit 64
fi

source_path="$1"
name="$2"
target=".ref/${name}"

if [[ ! -e "$source_path" ]]; then
  echo "Source '$source_path' does not exist." >&2
  exit 1
fi

mkdir -p .ref "$target"

if [[ -d "$source_path" ]]; then
  rsync -a --delete "${source_path%/}/" "$target/"
else
  mkdir -p "$target"
  cp "$source_path" "$target/"
fi

echo "Reference copy is available at $target"
