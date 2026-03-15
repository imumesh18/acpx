# SPEC

## Product

`acpx` is a thin Rust client for launching ACP-compatible agent servers as
local subprocesses and talking to them through the official ACP Rust SDK.

The crate stays close to upstream ACP naming, request and response types, and
lifecycle rules. Its job is to remove repeated client-side boilerplate around:

- spawning an ACP subprocess,
- wiring child stdio into `ClientSideConnection`,
- owning the ACP I/O task and subprocess cleanup,
- exposing a small launch contract for known servers, and
- shipping a typed, committed snapshot of the official ACP registry.

`acpx` is pre-`1.0.0`, so the public API may still change as the transport and
installer boundaries are refined.

## Design Constraints

- Stay close to ACP and the official Rust SDK.
- Keep the public API runtime agnostic.
- Make the upstream SDK's local `!Send` task model explicit through
  `RuntimeContext`.
- Prefer typed errors over opaque error strings.
- Keep the dependency graph small and the crate easy to test offline.
- Avoid hidden policy around authentication, persistence, retry, or install
  flows.

## Public Modules

### `acpx`

`Connection` is the subprocess-backed ACP client handle.

Current responsibilities:

- spawn a local subprocess and pipe stdin and stdout into the upstream ACP SDK,
- own the `ClientSideConnection`,
- keep the ACP I/O task alive through a caller-provided local task spawner,
- forward upstream ACP methods without renaming their semantics,
- capture `session/update` notifications, and
- close idempotently by dropping ACP state, terminating the child, and waiting
  for process exit.

Current forwarded ACP methods:

- `initialize`
- `authenticate`
- `new_session`
- `load_session`
- `set_session_mode`
- `prompt`
- `cancel`
- `list_sessions`
- `set_session_config_option`
- `ext_method`
- `ext_notification`

Current observers:

- `subscribe()` returns the upstream ACP stream receiver.
- `subscribe_session_updates()` returns captured `session/update`
  notifications.

### `agent_server`

`agent_server` defines the launch contract for ACP servers known to `acpx`.

Current public pieces:

- `AgentServerMetadata` stores `id`, `name`, `description`, `version`, and an
  optional `icon`.
- `CommandSpec` stores the subprocess program, args, env overrides, and
  optional spawn directory.
- `AgentServer` provides metadata access plus `connect(runtime)` and a default
  `close(connection)` that delegates to `Connection::close()`.
- `CommandAgentServer` implements `AgentServer` for a fixed subprocess command.

Behavioral guarantees:

- ACP traffic uses child stdin and stdout only.
- Stderr is not part of ACP correctness.
- Dropping or closing a live `Connection` does not intentionally leave the
  child running.
- Missing launchers such as `npx` or `uvx` surface as typed errors.

### `agent_servers`

`agent_servers` is the generated catalog layer built from the official ACP
registry snapshot committed in the repository.

Current public pieces:

- `Server`, which implements `AgentServer`,
- lookup helpers: `all`, `get`, and `require`,
- platform helpers: `HostPlatform`, `host_platform`, `host_binary_target`, and
  `binary_target_for`,
- registry metadata accessors for repository, authors, license, and
  distribution details.

Current launch rules:

- Package-backed registry entries using `npx` or `uvx` are directly
  connectable.
- `npx` launches are non-interactive through `--yes`.
- Binary-only registry entries remain discoverable in the catalog but
  `connect(...)` returns `UnsupportedLaunch::BinaryDistribution`.
- The library preserves official ACP registry ids; aliasing belongs in consumer
  code.

Generation rules:

- Registry sync is a maintainer action, not a build-time network fetch.
- The generated Rust source is committed so normal builds remain offline and
  deterministic.

### `runtime`

The public runtime surface is:

- `Task<'a, T>`
- `LocalTask<T>`
- `RuntimeContext`

`RuntimeContext` is the explicit bridge between `acpx` and the caller's
runtime. It only requires the ability to spawn local `!Send` tasks.

### `error`

The crate-wide error surface is typed.

`Error` currently covers:

- unsupported launch modes,
- missing launchers,
- subprocess spawn and stdio capture failures,
- ACP protocol failures,
- closed-connection use,
- subprocess kill and wait failures,
- unexpected ACP I/O task termination.

`agent_servers::Error` currently covers:

- unknown registry ids,
- unsupported host platforms,
- missing binary targets for a known host mapping.

## Connection Lifecycle

The supported happy path is:

1. Resolve an `AgentServer` directly or through `agent_servers`.
2. Call `connect(runtime)` to spawn the subprocess and create the ACP client
   connection.
3. Call `initialize`.
4. Optionally call `authenticate` if the agent advertises auth methods.
5. Create or load a session with a session `cwd`.
6. Optionally apply session mode or config options.
7. Send prompts and observe streamed session updates.
8. Call `close()` when finished.

ACP rules that `acpx` preserves:

- initialization happens before session creation,
- omitted capabilities remain omitted capabilities,
- the session `cwd` is ACP session state and is distinct from the subprocess
  spawn directory in `CommandSpec`,
- stdout must remain ACP-clean newline-delimited JSON-RPC.

## Supported Behavior

- Local subprocess agents over stdio only.
- Runtime-neutral public API with explicit local `!Send` task spawning.
- Manual command-backed servers and generated registry-backed servers.
- Typed lookup and host-platform helpers for the committed registry snapshot.
- An example CLI in `examples/cli.rs` for manual integration testing.

## Deferred Behavior

The current contract does not include:

- non-stdio transports,
- automatic download, extraction, caching, or updates for registry binaries,
- runtime registry refresh,
- reconnect or retry policy,
- implicit session persistence,
- higher-level authentication UX beyond upstream ACP methods,
- abstractions that replace ACP request and response types with a separate
  conversation model.
