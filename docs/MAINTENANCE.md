# Maintenance

## Quality gates

- `just ci`
- `typos`
- `just fmt-check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked`
- `cargo nextest run --all --all-features --locked --no-tests pass`
- `cargo test --doc --all-features --locked`
- `cargo deny check`
- `cargo build --all --all-features --locked`

## Working defaults

- Make it work first, then make it beautiful, then make it fast.
- Leave the codebase better than you found it.
- Prefer simple code over clever code.
- Plan and spec ACP behavior before implementing it.
- Before `1.0.0`, prefer a better contract over preserving an early draft API.
- Treat API sketches in planning docs as illustrative until the implementation
  proves the final shape.
- Write meaningful tests from the spec and add only the level of testing that
  buys real confidence.
- Minimize third-party dependencies; copy or internalize code only when the
  maintenance tradeoff is clearly better than depending on a crate.
- Remove setup code that stops matching the intended ACP product direction.

## Reference code

- `.ref/` is reserved for cloned or copied upstream implementations used for
  research, comparison, or selective internalization.
- `.ref/` is ignored by git and must never be committed.
- Use `just ref-clone <url> [name]` to clone or refresh a git repository into
  `.ref/`.
- Use `just ref-copy <source> <name>` to copy a local dependency or checkout
  into `.ref/`.

## Versioning and commits

- The repository baseline starts at `0.0.1` as the first intentional release.
- The first repo setup commit is `init: abracadabra`.
- Regular work follows conventional commits.
- Commit messages and bodies should explain the intent of the change.
- Keep history linear and modular with unit commits that are independently
  useful, working, and reviewable.
- Split feature work into small ordered commits so each step lands with the
  code, tests, and docs required for that step.
- Release prep commits use `chore(release): vX.Y.Z`, and tags use `vX.Y.Z`.

## Release checklist

1. Enter the repo shell with `devenv shell` or `direnv allow`.
2. Run `just release [version]` from a clean worktree. Omit `version` to accept
   the next version suggested by `git-cliff`.
3. Review the generated `CHANGELOG.md`, release commit, and annotated tag.
   `CHANGELOG.md` is intentionally excluded from Oxfmt so git-cliff spacing
   stays stable.
4. Optional: run the release workflow manually with `dry_run=true` to validate
   the tag, changelog, release notes, and `cargo publish --dry-run` before the
   real publish.
5. Push with `git push origin HEAD --follow-tags`.
6. Confirm the tag-triggered workflow publishes through crates.io trusted
   publishing and updates the GitHub Release.

## CI

- `ci.yml` runs spell check, format, clippy, docs, nextest, doctests, build,
  and MSRV checks.
- `audit.yml` runs `cargo deny check`.
- `publish.yml` verifies the tag matches `Cargo.toml`, validates the checked-in
  changelog against `git-cliff`, always runs `cargo publish --dry-run`,
  uploads rendered release notes as an artifact, and publishes to crates.io
  plus updates the GitHub Release on tag push using crates.io trusted
  publishing.
