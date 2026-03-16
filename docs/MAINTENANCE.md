# Maintenance

## Local Quality Gates

Enter the repo shell with `devenv shell` or `direnv allow`. `just` is the
human-facing command surface, and each recipe forwards to a `devenv` task so
local runs and CI share the same environment and tooling.

Use `just quality` as the default local gate. It runs:

- `just fmt-check`
- `just lint`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked`
- `just test`
- `just example cli`
- `just audit`
- `just build`

`devenv test` is the matching raw `devenv` entrypoint. It runs the quality gate
and then the dedicated `test:check` task.

Useful single-purpose commands:

- `just fmt`
- `just fmt-check`
- `just lint`
- `just lint-fix`
- `just test`
- `just example cli`
- `just quality-fix`
- `just changelog`
- `just release-notes [version]`
- `just next-version`
- `just registry-sync`
- `just publish-dry-run`
- `devenv test`

## CI Workflows

GitHub Actions is split by responsibility:

- `ci.yml` installs Nix, enables the public `devenv` Cachix cache, installs
  `devenv`, runs `devenv tasks run quality:check`, runs a release build, and
  runs the MSRV check by overriding the Rust channel from `Cargo.toml`.
- `audit.yml` is reserved for scheduled or manual `cargo deny` runs through
  `devenv` tasks.
- `publish.yml` runs release validation, release note rendering, crates.io
  publish, and GitHub Release updates through `devenv` tasks.

The example CLI smoke test is part of `just quality`, so it also runs via
`devenv tasks run quality:check` in `ci.yml`.

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
