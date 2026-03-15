//! `acpx` is a thin Rust wrapper around the official Agent Client Protocol
//! (ACP) Rust SDK.
//!
//! The crate is pre-`1.0.0` and the ACP-facing API is still settling. The
//! current modules establish the runtime and error contracts used by the
//! connection, registry, and agent server work that follows.

pub mod acpx;
pub mod agent_server;
pub mod agent_servers;
pub mod error;
pub mod runtime;

pub use crate::acpx::Connection;
pub use crate::agent_server::{AgentMetadata, AgentServer, CommandAgentServer, CommandSpec};
pub use crate::error::{Error, Result, UnsupportedLaunch};
pub use crate::runtime::{LocalTask, RuntimeContext, Task};
