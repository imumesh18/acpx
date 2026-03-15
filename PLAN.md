# PLAN

## Current objective

Finish repository setup for an ACP-focused Rust library, then write the product
spec and execution plan before starting ACP feature implementation.

## Current state

- The crate baseline is `0.0.1` as the first intentional release.
- The crate builds, lints, documents, and tests cleanly.
- Local development uses `devenv` for tool provisioning and `just` as the
  command surface.
- `git-cliff` drives both `CHANGELOG.md` generation and GitHub Release notes.
- `.ref/` is reserved for ignored upstream reference code and local copied
  implementations.
- CI covers formatting, clippy, docs, tests, audit, and automated release
  publishing.
- The repository intent is now ACP-focused, but ACP feature work has not
  started.
- The current crate surface is intentionally minimal until the ACP-facing spec
  is written.
- The intended users include CLI, TUI, desktop, and mobile app builders, plus
  teams building orchestration engines and agentic platforms.

## Next updates

- Expand `SPEC.md` into a real ACP-facing contract before implementation work.
- Decide the first stable module boundaries for the thin client, `agent_server`,
  and registry support.
- Define what problems `acpx` solves better than using the ACP Rust SDK and
  registry directly.
- Keep `cliff.toml`, release automation, and docs aligned if the release flow
  changes.
- Keep this file short and update it when work meaningfully changes direction.
