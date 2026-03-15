# acpx

`acpx` is a simple Rust library and tooling for building on Agent Client
Protocol (ACP).

The repo starts at version `0.0.1` as the first intentional release.

## Status

- This repository is still in setup and planning mode.
- We are writing the product contract and development workflow before starting
  ACP feature implementation.
- The current Rust code should be treated as provisional scaffolding, not as the
  final public shape of the library.

## Product Direction

The working direction for `acpx` is:

- A thin client built on top of the ACP Rust SDK.
- Supporting modules such as `agent_server` and a registry client backed by the
  ACP registry.
- Better ergonomics for consumers than using the upstream pieces directly,
  while still staying close to the ACP model.
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

The repo currently contains setup, release automation, and planning material.
We are not committing to a stable ACP-facing API yet.

## Not Yet For Use

Do not consume `acpx` as a published SDK yet. Until the ACP-facing spec exists
and the first intentional release is cut, this repo is a design and setup
workspace.

## Future Installation

```toml
[dependencies]
acpx = "0.0.1"
```

## Development

Use `devenv shell` for a reproducible local shell, or `direnv allow` if you use
`direnv`. The shell provides `just`, `git-cliff`, `cargo-nextest`,
`cargo-deny`, `gh`, and `jq`.

Common commands:

- `just ci` runs the full local quality gate.
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
- Release preparation commits use `chore(release): vX.Y.Z`.
