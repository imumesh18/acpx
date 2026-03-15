# Maintenance

## Local Quality Gates

Use `just ci` as the default local gate. It currently runs:

- `typos`
- `cargo fmt --all -- --check`
- Oxfmt checks for Markdown, TOML, and YAML
- `cargo clippy --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked`
- `cargo nextest run --all --all-features --locked --no-tests pass`
- `cargo test --doc --all-features --locked`
- `cargo test --example cli --all-features --locked`
- `cargo deny check`
- `cargo build --all --all-features --locked`

Useful single-purpose commands:

- `just fmt`
- `just fmt-check`
- `just registry-sync`
- `just example-test`
- `just publish-dry-run`

## CI Workflows

GitHub Actions is split by responsibility:

- `ci.yml` runs `typos`, Rust and Markdown formatting checks, clippy, rustdoc,
  nextest, doctests, a release build, and the MSRV check.
- `audit.yml` runs `cargo deny check`.
- `publish.yml` verifies the release tag against `Cargo.toml`, checks that
  `CHANGELOG.md` matches `git-cliff`, runs `cargo publish --locked --dry-run`,
  renders release notes, and publishes to crates.io and GitHub Releases on a
  real tag push.

The example CLI smoke test is part of the local `just ci` gate. It is not
currently part of `ci.yml`.

## Registry Catalog Maintenance

- `src/agent_servers.rs` is generated. Do not edit it manually.
- Run `just registry-sync` to refresh the committed ACP registry snapshot.
- Keep generated output committed so normal builds remain offline and
  deterministic.

## Release Checklist

1. Enter the repo shell with `devenv shell` or `direnv allow`.
2. Run `just release [version]` from a clean worktree. Omit `version` to accept
   the next version suggested by `git-cliff`.
3. Review the generated `CHANGELOG.md`, release commit, and annotated tag.
   `CHANGELOG.md` stays outside Oxfmt so `git-cliff` formatting remains stable.
4. Optional: trigger `publish.yml` manually with `dry_run=true` to validate the
   tag, changelog, release notes, and `cargo publish --dry-run`.
5. Push with `git push origin HEAD --follow-tags`.
6. Confirm the tag-triggered workflow published to crates.io and updated the
   GitHub Release.

## Working Defaults

- Keep docs in sync with code and CI.
- Prefer simple code, typed errors, and deterministic tests.
- Keep ACP behavior spec-driven: update `SPEC.md` before expanding the public
  surface.
- Use `.ref/` for uncommitted reference code only.
