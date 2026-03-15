set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

doc:
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked

nextest:
  cargo nextest run --all --all-features --locked --no-tests pass

doctest:
  cargo test --doc --all-features --locked

audit:
  cargo deny check

build:
  cargo build --all --all-features --locked

typos:
  typos

ci:
  ./scripts/quality-gates.sh

changelog:
  git cliff --config cliff.toml --output CHANGELOG.md

release-notes version='':
  if [ -n "{{version}}" ]; then \
    git cliff --config cliff.toml --tag "v{{version}}" --strip header; \
  else \
    git cliff --config cliff.toml --current --strip header; \
  fi

next-version:
  @git cliff --config cliff.toml --bumped-version | sed 's/^v//'

ref-clone url name='':
  ./scripts/ref-clone.sh {{url}} {{name}}

ref-copy source name:
  ./scripts/ref-copy.sh {{source}} {{name}}

publish-dry-run:
  cargo publish --locked --dry-run

release version='':
  ./scripts/release.sh {{version}}
