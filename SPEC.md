# SPEC

## Product intent

`acpx` is a thin Rust client library for launching ACP-compatible agent
servers and talking to them through the official ACP Rust SDK.

The crate is not intended to replace ACP concepts with a new abstraction
model. Its job is to remove the repetitive client-side boilerplate around:

- spawning a local ACP agent process,
- wiring stdio into `ClientSideConnection`,
- driving the ACP I/O task,
- exposing a small `AgentServer` contract for known agent definitions, and
- shipping a typed snapshot of the official ACP registry.

The library should stay close to upstream ACP naming, request/response types,
and lifecycle rules so consumers can move between `acpx` and the raw ACP SDK
without conceptual friction.

## Primary reference sources

These are the authoritative references for the first implementation plan:

- ACP Rust SDK `ClientSideConnection` and example client:
  <https://github.com/agentclientprotocol/rust-sdk>
- ACP protocol docs:
  - Initialization: <https://agentclientprotocol.com/protocol/initialization>
  - Session setup: <https://agentclientprotocol.com/protocol/session-setup>
  - Transports: <https://agentclientprotocol.com/protocol/transports>
  - Session modes: <https://agentclientprotocol.com/protocol/session-modes>
- ACP protocol repository and schema:
  <https://github.com/agentclientprotocol/agent-client-protocol>
- ACP registry:
  - Repo: <https://github.com/agentclientprotocol/registry>
  - CDN index: <https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json>
  - Schema: <https://cdn.agentclientprotocol.com/registry/v1/latest/agent.schema.json>

## Problem statement

Using the ACP Rust SDK directly is already possible, but a client author still
has to solve the same mechanical problems each time:

- choose and spawn an agent command,
- connect child stdin/stdout to `ClientSideConnection`,
- keep the ACP I/O future alive,
- manage process shutdown,
- surface registry metadata in a typed way,
- write ad hoc glue for simple local testing.

`acpx` should solve those problems while keeping the actual ACP protocol
surface recognizable and mostly unchanged.

## First release contract

The first ACP-focused release of `acpx` should provide:

- a thin `acpx` module on top of `agent_client_protocol::ClientSideConnection`,
- an `agent_server` module with a small trait-based contract for launchable ACP
  agent definitions,
- an `agent_servers` module backed by a generated snapshot of the official ACP
  registry and preserving the official registry ids,
- an example CLI in `examples/cli.rs` for local integration testing and manual
  protocol verification.

The first release is explicitly scoped to **local subprocess agents over
stdio**. ACP transport drafts such as streamable HTTP are out of scope.

## Stability stance

`acpx` is pre-`1.0.0`. Backward compatibility is not a design constraint for
this phase.

Before `1.0.0`, the project should prefer a better contract over preserving an
early draft API. If implementation work shows that a sketched interface is
awkward, misleading, or too limiting, it should be changed rather than carried
forward for compatibility reasons.

## Design principles

- Stay close to ACP and the official Rust SDK.
- Keep the public API runtime agnostic.
- Prefer typed errors over `anyhow` in library code.
- Minimize external dependencies and justify each one.
- Avoid hidden policy. Authentication, retry, persistence, and install flows
  should remain visible to consumers unless the wrapper meaningfully reduces
  repeated boilerplate.
- Examples may choose a concrete runtime for convenience; the library API may
  not require one.

## Public module contract

### `acpx`

`acpx` is the ergonomic entrypoint for consumers. It should wrap ACP
connection setup, not ACP semantics.

Responsibilities:

- create a client-side ACP connection from a spawned agent subprocess,
- wire stdio into the upstream `ClientSideConnection`,
- start the ACP I/O driver using a caller-provided task spawner or equivalent
  runtime-neutral hook,
- expose the connected ACP handle in a way that still uses upstream ACP
  requests, responses, and capabilities,
- own shutdown ordering for connection teardown and child-process cleanup.

Required behavior:

- callers must be able to reach `initialize`, `authenticate`, `new_session`,
  `load_session`, `prompt`, `cancel`, and other upstream ACP methods without
  `acpx` renaming or materially changing them,
- callers must be able to observe streamed ACP session updates,
- the wrapper must not hide protocol negotiation details such as
  `protocolVersion`, agent capabilities, session modes, or `authMethods`.

Non-goals for this module:

- no alternate conversation model,
- no opinionated retry or reconnection policy,
- no implicit session persistence,
- no transport support beyond stdio in the first release.

### `agent_server`

`agent_server` defines what it means for `acpx` to know how to launch a
specific ACP agent.

The intended design target is a small trait with metadata accessors plus an
async `connect` operation:

```rust
pub trait AgentServer: Send + Sync {
    fn id(&self) -> SharedString;
    fn icon(&self) -> SharedString;
    fn name(&self) -> SharedString;
    fn description(&self) -> SharedString;
    fn version(&self) -> SharedString;

    fn connect(&self) -> Task<'_, Result<AgentServerConnectResult>>;

    fn close(&self, connection: AgentServerConnectResult) -> Task<'_, Result<()>> {
        Box::pin(async move { connection.close().await })
    }
}
```

This shape is intentionally illustrative, not final.

The intent is fixed:

- an `AgentServer` value represents a concrete, user-selectable ACP agent,
- `connect` launches the agent and returns a connected ACP client handle plus
  the resources needed to shut it down cleanly,
- `close` performs orderly shutdown and defaults to delegating to the returned
  connection handle.

The exact trait contract is **not** fixed yet. During implementation, it is
acceptable to replace this sketch with a better API if that improves lifecycle
ownership, runtime-agnostic process handling, error clarity, or overall
ergonomics. The important constraint is preserving the intent of a thin,
simple launch contract rather than preserving this exact method signature set.

`AgentServerConnectResult` should contain, directly or indirectly:

- the ready-to-use ACP client connection,
- ownership of the spawned child process,
- the state required to stop the background ACP I/O task,
- enough lifecycle hooks to guarantee child processes are not leaked.

Behavioral requirements:

- ACP messages travel over child stdin/stdout only,
- stderr is treated as logs or diagnostic output and must never be required for
  ACP correctness,
- shutdown must be idempotent and best-effort safe,
- dropping a live connection must not leave an orphaned agent process behind.

Runtime-agnostic requirement:

- the public async API must not expose Tokio types,
- the crate should prefer a small future alias such as `Task<'a, T>` over
  macro-driven runtime-specific traits,
- if process management requires an external crate, it must be runtime-neutral.

### `agent_servers`

`agent_servers` provides a compile-time Rust view of the official ACP registry.

Responsibilities:

- expose typed lookup and iteration over known ACP agents,
- preserve the official registry identity fields: `id`, `name`, `version`,
  `description`, `repository`, `authors`, `license`, `icon`, and
  `distribution`,
- select launch metadata using the official registry distribution model:
  `binary`, `npx`, and `uvx`,
- generate Rust source from the official registry JSON through an explicit
  repo-local maintenance script.

Generation rules:

- registry code generation is an explicit maintainer action, not a build-time
  network fetch,
- the generator fetches the official registry JSON from the ACP CDN and writes
  generated Rust source into the repository,
- the generated source is committed so normal crate builds stay offline and
  deterministic,
- generated `npx` launches should be non-interactive so first-run installs do
  not block on confirmation prompts.

Connection rules:

- registry-backed agents should implement the shared `AgentServer` contract,
- direct launch support is guaranteed only for distributions that are already
  directly invocable on the host, specifically `npx` and `uvx`,
- binary archive installation and extraction are not part of the first release,
  so binary-only registry entries remain discoverable but may return a typed
  unsupported-launch error from `connect`.

This keeps the `agent_servers` module simple and honest. It provides real value for
lookup, listing, inspection, and launch of package-backed agents without
turning the crate into an installer or package manager.

Consumer-facing naming policy is explicitly out of scope for the library. If a
CLI or application wants friendlier ids such as `droid` instead of
`factory-droid`, that mapping belongs in the consumer layer, not in the core
`acpx::agent_servers` API.

### `examples/cli.rs`

The example CLI is an internal testing harness, not a published product
surface.

Purpose:

- manually verify that `acpx` can connect to real ACP agents,
- run a prompt loop against a selected agent,
- inspect initialization results, auth methods, capabilities, and session
  updates in a terminal,
- exercise shutdown behavior and agent-server lookups during development.

Requirements:

- place it in `examples/cli.rs`,
- keep its interface intentionally simple and disposable,
- allow it to define its own input aliases over raw registry ids when that
  improves ergonomics for manual testing,
- allow it to use a concrete async runtime if that keeps the example small,
  even though the library remains runtime agnostic.

The example CLI exists to dogfood the library and validate real integration
flows before stabilizing the public API.

## Connection lifecycle

The intended happy-path lifecycle is:

1. Choose an `AgentServer` directly or through `agent_servers`.
2. Launch the agent as a local subprocess that speaks ACP over stdio.
3. Create the upstream `ClientSideConnection`.
4. Start the ACP I/O driver.
5. Call `initialize` with client info and client capabilities.
6. Optionally call `authenticate` if the chosen agent requires it.
7. Create or load a session with an absolute `cwd`.
8. Send prompts and receive streamed session updates.
9. Close the ACP connection and terminate or reap the child process.

Important ACP rules that `acpx` must preserve:

- initialization is mandatory before session creation,
- omitted capabilities mean unsupported capabilities,
- the working directory passed to `session/new` or `session/load` is session
  state and must not be confused with the agent process spawn directory,
- stdio is newline-delimited JSON-RPC and stdout must stay ACP-clean.

## Error model

The library should use typed errors that make these categories distinct:

- agent-server catalog lookup or platform resolution failure,
- unsupported launch strategy for a known registry entry,
- missing local launcher prerequisite such as `npx` or `uvx`,
- subprocess spawn failure,
- stdio transport failure,
- ACP protocol or JSON-RPC failure,
- shutdown and cleanup failure.

The public library should not normalize all failures into a single opaque error
string.

## Explicitly deferred

The following are intentionally outside the first spec:

- automatic download, caching, extraction, and update management for registry
  binary distributions,
- higher-level authentication UX abstractions beyond exposing upstream ACP auth
  methods and requests,
- non-stdio ACP transports,
- a stable installer story,
- registry refresh or sync at runtime,
- custom retry, reconnection, or resilience policies,
- any abstraction that hides ACP requests and responses behind a separate
  conversational model.

## Current code status

The first ACP-focused vertical slice is implemented:

- `acpx::Connection` launches local subprocess agents over stdio, forwards the
  upstream ACP methods, captures `session/update` notifications, and owns
  shutdown.
- `agent_server` exposes a handwritten runtime-neutral launch contract plus a
  manual command-backed server.
- `agent_servers` is generated from the official ACP registry, committed to
  the repository, and provides raw lookup plus typed host-platform helpers.
- `examples/cli.rs` provides a single-shot CLI harness for manual integration
  testing.

The crate remains pre-`1.0.0`, so the public API may still change as follow-up
work improves the transport boundary, installer story, and overall ergonomics.
