# SPEC

## Product intent

`acpx` is intended to become a simple Rust library and tooling layer for
building on Agent Client Protocol (ACP). The library should stay thin,
ergonomic, and close to the upstream ACP model rather than becoming a large
abstraction layer.

The current plan is to build on top of the official ACP Rust SDK and add a
small set of supporting modules where the user experience can be improved
without hiding core ACP concepts.

## Primary reference sources

- ACP Rust SDK: <https://github.com/agentclientprotocol/rust-sdk>
- ACP protocol: <https://github.com/agentclientprotocol/agent-client-protocol>
- ACP registry: <https://github.com/agentclientprotocol/registry>
- ACP introduction: <https://agentclientprotocol.com/get-started/introduction>

## Target scope

The first planning pass should converge on these areas:

- A thin ACP client surface for application authors.
- An `agent_server` support module for working with ACP-compatible agent
  servers.
- A registry module that can work with the official ACP registry and present
  typed data to callers.
- Supporting types, transport helpers, and error handling needed to make the
  user experience better than raw protocol access.
- A library shape that works well for CLI, TUI, desktop, and mobile apps, as
  well as agent orchestration engines and broader agentic platforms.

## Design constraints

- Stay close to the ACP protocol and upstream Rust SDK semantics.
- Prefer simple APIs with typed errors and minimal dependencies.
- Keep durable behavior in code, tests, CI, and repo docs.
- Spec first, then implementation, then optimization.
- Tests should be spec-driven and only added where they provide real
  confidence.

## Explicitly deferred

These items are intentionally not specified yet:

- Final module names and exact public API boundaries.
- Transport, auth, and retry policy decisions.
- Registry caching, sync policy, or offline behavior.
- Runtime requirements such as Tokio-only versus broader compatibility.
- Any public API unrelated to the ACP product direction.

## Current code status

The repository currently contains setup scaffolding only. No ACP-facing public
API is committed yet, and no pre-ACP exploratory API should be treated as part
of the product direction.
