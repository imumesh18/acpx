//! `acpx` is a thin Rust wrapper around the official Agent Client Protocol
//! (ACP) Rust SDK.
//!
//! The crate is pre-`1.0.0` and the ACP-facing API is still settling. The
//! current modules provide:
//!
//! - `Connection` for subprocess-backed ACP sessions over stdio,
//! - `AgentServer` and `CommandAgentServer` for launchable agent definitions,
//! - a generated ACP registry snapshot plus raw-id `registry` helpers,
//! - runtime-neutral local task hooks for the upstream SDK's `!Send` model.

pub mod acpx;
pub mod agent_server;
pub mod agent_servers;
pub mod error;
pub mod registry;
pub mod runtime;

pub use crate::acpx::Connection;
pub use crate::agent_server::{AgentMetadata, AgentServer, CommandAgentServer, CommandSpec};
pub use crate::agent_servers::{
    RegistryAgentServer, RegistryBinaryTarget, RegistryDistribution, RegistryPackageDistribution,
    RegistryPackageManager,
};
pub use crate::error::{Error, Result, UnsupportedLaunch};
pub use crate::registry::{HostPlatform, RegistryError};
pub use crate::runtime::{LocalTask, RuntimeContext, Task};
