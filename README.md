# acpx

`acpx` is a simple Rust library and tooling for building on Agent Client
Protocol (ACP).

The repo starts at version `0.0.1` as the first intentional release.

## Status

- The first ACP-focused vertical slice is implemented.
- The crate currently ships a subprocess-backed `Connection`, a handwritten
  `AgentServer` contract, a generated ACP registry snapshot, a raw-id
  `registry` module, and a single-shot example CLI.
- `SPEC.md` records the intended contract and `PLAN.md` records the completed
  delivery plan plus follow-up notes.
- The current Rust code is usable for local experimentation, but the public API
  is still provisional.
- Before `1.0.0`, breaking changes are acceptable if they produce a better
  contract.

## Product Direction

The working direction for `acpx` is:

- A thin client built on top of the ACP Rust SDK.
- Supporting modules such as `agent_server` and a registry client backed by the
  ACP registry.
- Better ergonomics for consumers than using the upstream pieces directly,
  while still staying close to the ACP model.
- Stdio subprocess agents first; broader transport and installer stories are
  deferred.
- Intended for teams building CLI, TUI, desktop, and mobile apps, plus agent
  orchestration engines and broader agentic platforms.
- Minimal dependencies, typed errors, and simple APIs that are easy to test and
  reason about.

## Reference Sources

These are the current primary references for planning:

- ACP Rust SDK: <https://github.com/agentclientprotocol/rust-sdk>
- ACP protocol: <https://github.com/agentclientprotocol/agent-client-protocol>
- ACP registry: <https://github.com/agentclientprotocol/registry>
- ACP introduction docs: <https://agentclientprotocol.com/get-started/introduction>

## Current Scope

- Local ACP subprocess agents over stdio.
- Package-backed registry entries launched via `npx` or `uvx`.
- Binary-only registry entries that remain discoverable but return a typed
  unsupported-launch error in v0.
- Library registry helpers that preserve the official ACP registry ids.
- A disposable example CLI in [`examples/cli.rs`](examples/cli.rs) for manual
  integration testing.

## Still Changing

Do not treat `acpx` as a stable published SDK yet. The crate is intentionally
pre-`1.0.0`, the installer story for binary registry distributions is deferred,
and the transport boundary may still be refined.

## Installation

```toml
[dependencies]
acpx = "0.0.1"
```

## Example CLI

```sh
cargo run --example cli -- codex "Summarize this repository" --cwd "$(pwd)"
```

The example CLI accepts raw registry ids and a small CLI-local alias table, so
inputs such as `codex`, `droid`, and `vibe` resolve to the official
registry ids before launch. It then initializes the agent, creates a session,
optionally applies `--mode` and `--permission-mode`, prints streamed
`session/update` notifications, and closes cleanly.

## Development

Use `devenv shell` for a reproducible local shell, or `direnv allow` if you use
`direnv`. The shell provides `node`, `npx`, `just`, `git-cliff`, `cargo-nextest`,
`cargo-deny`, `jq`, and the Rust toolchain.

Common commands:

- `just fmt` formats Rust, Markdown, YAML, and TOML files.
- `just fmt-check` checks Rust, Markdown, YAML, and TOML formatting.
- `just ci` runs the full local quality gate.
- `just registry-sync` refreshes `src/agent_servers.rs` from the official ACP
  registry snapshot.
- `just example-test` runs the example CLI smoke test.
- `just typos` runs source and docs spell checking.
- `just changelog` regenerates `CHANGELOG.md` from the current git history.
- `just next-version` prints the next version suggested by `git-cliff`.
- `just ref-clone <url> [name]` clones or refreshes upstream reference code in
  `.ref/`.
- `just ref-copy <source> <name>` copies local reference code into `.ref/`.
- `just release [version]` updates `Cargo.toml`, refreshes `CHANGELOG.md`, runs
  the quality gate, creates a release commit, and tags `vX.Y.Z`.

## Working Agreement

- Make it work first, then make it beautiful, then make it fast.
- Leave the codebase better than you found it.
- Prefer simple code over clever code.
- Plan and spec first for ACP behavior; implementation follows the written
  contract.
- Treat draft API sketches as intent, not as a compatibility promise.
- Before `1.0.0`, prefer improving the contract over preserving a weak early
  API.
- Keep dependencies minimal and copy or internalize code only when the
  maintenance tradeoff is clearly better for the crate.
- Write meaningful, spec-driven tests that cover the behavior we actually care
  about.

## Reference Code

Use `.ref/` for cloned or copied upstream implementations while researching or
porting behavior. The folder is intentionally ignored by git and should never
be committed. Use the `just ref-clone` and `just ref-copy` helpers so this
stays consistent.

## Releases

Push the release commit and tag with `git push origin HEAD --follow-tags` after
`just release`. GitHub Actions then verifies the tag, performs a
`cargo publish --dry-run`, publishes the crate, and creates or updates the
GitHub Release using notes rendered from the same `git-cliff` configuration as
`CHANGELOG.md`.

Commit policy:

- The initial repo setup commit is `init: abracadabra`.
- Follow conventional commits for regular work.
- Write commit messages and bodies that explain intent.
- Prefer linear, modular unit commits that each work on their own and add a
  coherent slice of value.
- Split feature work into small ordered commits so each step includes the code,
  tests, and docs needed for that step.
- Release preparation commits use `chore(release): vX.Y.Z`.
