//! `acpx` is a thin Rust client for launching ACP-compatible agent servers and
//! talking to them through the official Agent Client Protocol (ACP) Rust SDK.
//!
//! The crate stays close to upstream ACP types and lifecycle rules. Its main
//! pieces are:
//!
//! - `Connection` for subprocess-backed ACP sessions over stdio,
//! - `AgentServer`, `CommandAgentServer`, and `CommandSpec` for launchable
//!   agent definitions,
//! - `agent_servers` for the generated ACP registry catalog, and
//! - `RuntimeContext` for the upstream SDK's local `!Send` task model.
//!
//! The current scope is local subprocess agents over stdio. Registry entries
//! backed by `npx` or `uvx` are directly connectable. Binary-only registry
//! entries remain discoverable but are not launchable in v0.
//!
//! ```rust,no_run
//! use acpx::{
//!     AgentServer, AgentServerMetadata, CommandAgentServer, CommandSpec, RuntimeContext,
//! };
//! use agent_client_protocol as acp;
//! use futures::executor::block_on;
//!
//! let runtime = RuntimeContext::new(|task| {
//!     block_on(task);
//! });
//!
//! let server = CommandAgentServer::new(
//!     AgentServerMetadata::new("codex-acp", "Codex CLI", "0.10.0")
//!         .description("ACP adapter for OpenAI's coding assistant"),
//!     CommandSpec::new("npx")
//!         .arg("--yes")
//!         .arg("@zed-industries/codex-acp@0.10.0"),
//! );
//!
//! block_on(async {
//!     let connection = server.connect(&runtime).await?;
//!     let _initialize = connection
//!         .initialize(
//!             acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
//!                 acp::Implementation::new("acpx-docs", "0.1.0").title("acpx Docs"),
//!             ),
//!         )
//!         .await?;
//!
//!     connection.close().await
//! })?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod acpx;
pub mod agent_server;
pub mod agent_servers;
pub mod error;
pub mod runtime;

pub use crate::acpx::Connection;
pub use crate::agent_server::{AgentServer, AgentServerMetadata, CommandAgentServer, CommandSpec};
pub use crate::agent_servers::{Error as AgentServerError, HostPlatform};
pub use crate::error::{Error, Result, UnsupportedLaunch};
pub use crate::runtime::{LocalTask, RuntimeContext, Task};
