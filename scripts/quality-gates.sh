#!/usr/bin/env bash
set -euo pipefail

typos
cargo fmt --all -- --check
npx --yes oxfmt@0.40.0 --no-error-on-unmatched-pattern --check \
  "**/*.md" "**/*.toml" "**/*.yml" "**/*.yaml" "!CHANGELOG.md"
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked
cargo nextest run --all --all-features --locked --no-tests pass
cargo test --doc --all-features --locked
cargo test --example cli --all-features --locked
cargo deny check
cargo build --all --all-features --locked
