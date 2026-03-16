# AGENT

Read this file first.

## Repo map

- `src/lib.rs`: crate entrypoint and crate-level documentation.
- `src/acpx.rs`: subprocess-backed ACP connection wrapper.
- `src/agent_server.rs`: handwritten launch contract and manual command server.
- `src/agent_servers.rs`: generated agent-server catalog, metadata, and platform helpers.
- `src/bin/registry-sync.rs`: registry snapshot generator.
- `devenv.nix`: canonical shell packages, local command definitions, and task hooks.
- `examples/cli.rs`: single-shot integration harness for real agents.
- `docs/GETTING_STARTED.md`: user-facing setup and usage guide.
- `docs/MAINTENANCE.md`: maintainer workflows and CI and release behavior.
- `.ref/`: ignored reference checkouts and copied upstream implementations.
- `SPEC.md`: current public contract for the crate.
- `PLAN.md`: forward-looking roadmap and open design work.

## Commands

- `just fmt`
- `just fmt-check`
- `just lint`
- `just lint-fix`
- `just test`
- `just example cli`
- `just quality`
- `just quality-fix`
- `just audit`
- `just build`
- `just doc`
- `just changelog`
- `just release-notes [version]`
- `just next-version`
- `just registry-sync`
- `just publish-dry-run`
- `just release [version]`
- `just ref-clone <url> [name]`
- `just ref-copy <source> <name>`
- `devenv shell`
- `devenv test`
- `devenv tasks run quality:check`

## Rules

- Keep the crate on Rust 2024 and honor the MSRV in `Cargo.toml`.
- Keep repo guidance minimal and delete stale context instead of preserving it.
- Keep docs in sync with code, tests, and CI behavior.
- Put durable behavior in code, tests, CI, or these root docs instead of chat.
- Keep tests deterministic and independent of network, time, ordering, and host state.
- Do not add `unsafe`; the manifest forbids it.
- Prefer typed errors and a small dependency graph.
- Use `.ref/` for ignored reference or copied upstream code; never commit it.
- Follow conventional commits.
- Commit messages and bodies should explain intent, not just the file changes.
- Keep history linear and modular: prefer unit commits that each stand on their
  own, pass checks, and deliver a coherent slice of value.
- Split feature work into ordered commits where each step is independently
  working and includes the code, tests, and docs needed for that step.
- Release prep commits use `chore(release): vX.Y.Z` and tags use `vX.Y.Z`.
- Treat the ACP protocol spec, ACP Rust SDK, ACP registry, and ACP docs as the
  primary external references when planning behavior.
- Do not start ACP feature implementation before the requested behavior is
  written down in `SPEC.md` and reflected in `PLAN.md`.
- Work in order: make it work, make it beautiful, make it fast.
- Leave the codebase better than you found it and avoid broken windows.
- Prefer simple code over clever code, and meaningful spec-driven tests over test count.
- Minimize external dependencies; prefer owned code when the maintenance tradeoff is justified.
