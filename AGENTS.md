# AGENT

Read this file first.

## Repo map

- `src/lib.rs`: crate entrypoint and crate-level documentation.
- `scripts/`: repo-local automation for quality gates and release preparation.
- `.ref/`: ignored reference checkouts and copied upstream implementations.
- `SPEC.md`: current public contract for the crate.
- `PLAN.md`: current execution plan and open follow-up work.
- `docs/MAINTENANCE.md`: quality gates, release checklist, and CI behavior.

## Commands

- `just ci`
- `typos`
- `just fmt`
- `just fmt-check`
- `just ref-clone <url> [name]`
- `just ref-copy <source> <name>`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --all --all-features --locked --no-tests pass`
- `cargo test --doc --all-features --locked`
- `cargo deny check`
- `cargo build --all --all-features --locked`

## Rules

- Keep the crate on Rust 2024 and honor the MSRV in `Cargo.toml`.
- Keep repo guidance minimal and update it only when the project actually grows.
- Put durable behavior in code, tests, CI, or these root docs instead of chat.
- Keep tests deterministic and independent of network, time, ordering, and host state.
- Do not add `unsafe`; the manifest forbids it.
- Prefer typed errors and a small dependency graph.
- Use `.ref/` for ignored reference or copied upstream code; never commit it.
- Follow conventional commits. The first repo setup commit is `init: abracadabra`.
- Commit messages and bodies should explain intent, not just the file changes.
- Keep history linear and modular: prefer unit commits that each stand on their
  own, pass checks, and deliver a coherent slice of value.
- Split feature work into ordered commits where each step is independently
  working and includes the code, tests, and docs needed for that step.
- Keep the repo baseline aligned with `0.0.1` as the first intentional release.
- Release prep commits use `chore(release): vX.Y.Z` and tags use `vX.Y.Z`.
- Treat the ACP protocol spec, ACP Rust SDK, ACP registry, and ACP docs as the
  primary external references when planning behavior.
- Do not start ACP feature implementation before the requested behavior is
  written down in `SPEC.md` and reflected in `PLAN.md`.
- Work in order: make it work, make it beautiful, make it fast.
- Leave the codebase better than you found it and avoid broken windows.
- Prefer simple code over clever code, and meaningful spec-driven tests over test count.
- Minimize external dependencies; prefer owned code when the maintenance tradeoff is justified.
