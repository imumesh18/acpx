# Getting Started

`acpx` is a thin wrapper around the ACP Rust SDK. You still work with ACP
requests and responses directly; `acpx` mainly handles subprocess launch, stdio
transport wiring, and connection lifecycle.

## What You Need

- A runtime that can run local `!Send` tasks.
- An ACP-compatible agent that can be launched as a local subprocess.
- An absolute working directory for ACP sessions.
- `npx` or `uvx` installed if you plan to launch package-backed registry
  entries.

`RuntimeContext` is the bridge between `acpx` and your runtime. The ACP SDK
uses local `!Send` tasks, so `acpx` makes that requirement explicit instead of
hiding it behind Tokio types.

## Runtime Setup

The library API is runtime neutral. If you use Tokio, run `acpx` on a local
task runner such as `LocalSet`:

```rust
use acpx::RuntimeContext;

let runtime = RuntimeContext::new(|task| {
    tokio::task::spawn_local(task);
});
```

That closure must only be used while a Tokio `LocalSet` is active.

## Launch a Fixed Command

Use `CommandAgentServer` when your application already knows the command line it
wants to run.

```rust
use acpx::{
    AgentServer, AgentServerMetadata, CommandAgentServer, CommandSpec, RuntimeContext,
};
use agent_client_protocol as acp;

async fn run(runtime: &RuntimeContext) -> Result<(), Box<dyn std::error::Error>> {
    let server = CommandAgentServer::new(
        AgentServerMetadata::new("codex-acp", "Codex CLI", "0.10.0")
            .description("ACP adapter for OpenAI's coding assistant"),
        CommandSpec::new("npx")
            .arg("--yes")
            .arg("@zed-industries/codex-acp@0.10.0"),
    );

    let connection = server.connect(runtime).await?;

    let _initialize = connection
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
                acp::Implementation::new("my-app", "0.1.0").title("My App"),
            ),
        )
        .await?;

    let session = connection
        .new_session(acp::NewSessionRequest::new(std::env::current_dir()?))
        .await?;

    let _response = connection
        .prompt(acp::PromptRequest::new(
            session.session_id.clone(),
            vec![String::from("Summarize this repository").into()],
        ))
        .await?;

    connection.close().await?;
    Ok(())
}
```

`CommandSpec::cwd(...)` sets the subprocess spawn directory. That is separate
from the session `cwd` sent through ACP in `session/new` or `session/load`.

## Use the Generated Registry Catalog

Use `agent_servers` when you want the committed ACP registry snapshot and its
official ids.

```rust
use acpx::{AgentServer, RuntimeContext, agent_servers};

async fn run(runtime: &RuntimeContext) -> Result<(), Box<dyn std::error::Error>> {
    let server = agent_servers::require("claude-acp")?;
    println!("launching {} {}", server.name(), server.version());

    let connection = server.connect(runtime).await?;
    connection.close().await?;
    Ok(())
}
```

Useful catalog helpers:

- `agent_servers::all()` to iterate the snapshot.
- `agent_servers::get(id)` for optional lookup.
- `agent_servers::require(id)` for typed lookup failure.
- `agent_servers::host_platform()` to map the current host to ACP registry
  targets.
- `agent_servers::host_binary_target(&server)` to inspect binary metadata for a
  known server.

## Supported and Unsupported Launches

- `npx` and `uvx` package distributions are launchable.
- Binary-only registry entries remain visible in the catalog for lookup and
  inspection.
- Binary-only entries are not launchable in v0. Calling `connect(...)` on them
  returns `UnsupportedLaunch::BinaryDistribution`.

## Observe Session Updates

`Connection::subscribe_session_updates()` returns a stream of captured
`session/update` notifications from the agent. Use it when you need to surface
streamed agent output or progress updates while a prompt is running.

## Manual Testing

The repo includes a single-shot CLI harness in `examples/cli.rs`:

```sh
cargo run --example cli -- codex "Summarize this repository" --cwd "$(pwd)"
```

Use it to inspect initialization results, session metadata, and streamed
updates against real agents while working on integrations.
