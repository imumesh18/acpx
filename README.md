# acpx

[![CI](https://github.com/imumesh18/acpx/actions/workflows/ci.yml/badge.svg)](https://github.com/imumesh18/acpx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/acpx.svg)](https://crates.io/crates/acpx)
[![Docs.rs](https://img.shields.io/docsrs/acpx)](https://docs.rs/acpx)
[![MSRV](https://img.shields.io/badge/rustc-1.85%2B-b7410e)](https://github.com/imumesh18/acpx/blob/main/Cargo.toml)
[![License](https://img.shields.io/crates/l/acpx)](https://crates.io/crates/acpx)

> [!IMPORTANT]
> This is an independent project and is **not** an official Agent Client
> Protocol (ACP) project.

`acpx` is a thin Rust client for launching Agent Client Protocol (ACP) agent
servers as local subprocesses and talking to them through the official ACP Rust
SDK.

The crate stays close to upstream ACP types and lifecycle rules. It removes the
repetitive client-side work around subprocess launch, stdio wiring, ACP I/O
task ownership, and typed access to the official ACP registry snapshot.

`acpx` is still pre-`1.0.0`, so the public API may change as the transport and
installer boundaries are refined.

## What `acpx` Covers

- Local ACP subprocesses over stdio.
- A subprocess-backed `Connection` that forwards upstream ACP methods.
- A handwritten `AgentServer` contract plus `CommandAgentServer` for fixed
  launch commands.
- A generated `agent_servers` catalog that preserves official ACP registry ids
  and metadata.
- Direct launch support for package-backed registry entries that use `npx` or
  `uvx`.
- Typed errors for launch, registry lookup, platform resolution, and ACP
  failures.

## What `acpx` Does Not Cover Yet

- Non-stdio ACP transports.
- Automatic download, extraction, or installation of registry binaries.
- Reconnection, persistence, or higher-level chat abstractions.
- Consumer-facing alias policy for registry ids.

Binary-only registry entries remain visible in `agent_servers`, but
`connect(...)` returns a typed unsupported-launch error in v0.

## Installation

```toml
[dependencies]
acpx = "0.1.0"
```

## Quick Start

1. Provide a `RuntimeContext` that can run local `!Send` tasks.
2. Choose either a manual `CommandAgentServer` or a generated
   `agent_servers::Server`.
3. Call `connect`, then `initialize`, `new_session` or `load_session`,
   `prompt`, and `close`.

Start with [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) for concrete
examples and runtime setup.

## Current Scope

- `Connection` forwards ACP methods such as `initialize`, `authenticate`,
  `new_session`, `load_session`, `set_session_mode`, `prompt`, and
  `set_session_config_option`.
- `subscribe()` exposes the raw ACP stream receiver from the upstream SDK.
- `subscribe_session_updates()` exposes captured `session/update`
  notifications.
- `agent_servers::{all, get, require}` expose the committed registry snapshot
  using official ACP ids.
- `agent_servers::{host_platform, host_binary_target}` help inspect registry
  binary metadata without adding an installer.

## Example CLI

```sh
cargo run --example cli -- codex "Summarize this repository" --cwd "$(pwd)"
```

The example CLI is a manual integration harness. It resolves raw registry ids
and a small alias table, initializes the selected agent, creates a session,
optionally applies `--mode` and `--permission-mode`, prints streamed
`session/update` notifications, and closes cleanly.

## Further Reading

- [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)
- [SPEC.md](SPEC.md)
- [PLAN.md](PLAN.md)
- [docs/MAINTENANCE.md](docs/MAINTENANCE.md)

## References

- ACP protocol: <https://github.com/agentclientprotocol/agent-client-protocol>
- ACP Rust SDK: <https://github.com/agentclientprotocol/rust-sdk>
- ACP registry: <https://github.com/agentclientprotocol/registry>
