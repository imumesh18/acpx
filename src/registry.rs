use std::env;

use thiserror::Error;

use crate::{
    agent_server::AgentServer,
    agent_servers::{
        REGISTRY_VERSION, RegistryAgentServer, RegistryBinaryTarget, registry_agent_server,
        registry_agent_servers,
    },
};

pub use crate::agent_servers::{
    RegistryDistribution, RegistryPackageDistribution, RegistryPackageManager,
};

/// Typed failures from registry lookup and host-platform helpers.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RegistryError {
    /// The generated registry snapshot does not contain the requested id.
    #[error("agent server `{id}` was not found")]
    UnknownAgentServer {
        /// The unresolved official ACP registry id.
        id: String,
    },

    /// The current or requested host target is outside the v0 platform map.
    #[error("unsupported host platform `{os}/{arch}`")]
    UnsupportedHostPlatform {
        /// The operating system identifier.
        os: String,
        /// The CPU architecture identifier.
        arch: String,
    },

    /// A binary-backed registry entry does not provide a build for the
    /// requested host target.
    #[error("agent server `{id}` does not publish a binary for host platform `{target}`")]
    MissingBinaryTarget {
        /// The registry id for the agent server.
        id: String,
        /// The official ACP registry target triple.
        target: String,
    },
}

/// Host platforms currently mapped to ACP registry binary targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostPlatform {
    DarwinAarch64,
    DarwinX86_64,
    LinuxAarch64,
    LinuxX86_64,
    WindowsAarch64,
    WindowsX86_64,
}

impl HostPlatform {
    /// Resolves a Rust target OS and architecture into an ACP registry target.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::UnsupportedHostPlatform`] when the pair does
    /// not map to a supported ACP registry target in v0.
    pub fn from_target(os: &str, arch: &str) -> Result<Self, RegistryError> {
        match (os, arch) {
            ("macos", "aarch64") => Ok(Self::DarwinAarch64),
            ("macos", "x86_64") => Ok(Self::DarwinX86_64),
            ("linux", "aarch64") => Ok(Self::LinuxAarch64),
            ("linux", "x86_64") => Ok(Self::LinuxX86_64),
            ("windows", "aarch64") => Ok(Self::WindowsAarch64),
            ("windows", "x86_64") => Ok(Self::WindowsX86_64),
            _ => Err(RegistryError::UnsupportedHostPlatform {
                os: os.to_owned(),
                arch: arch.to_owned(),
            }),
        }
    }

    /// Returns the official ACP registry target identifier.
    #[must_use]
    pub fn registry_target(self) -> &'static str {
        match self {
            Self::DarwinAarch64 => "darwin-aarch64",
            Self::DarwinX86_64 => "darwin-x86_64",
            Self::LinuxAarch64 => "linux-aarch64",
            Self::LinuxX86_64 => "linux-x86_64",
            Self::WindowsAarch64 => "windows-aarch64",
            Self::WindowsX86_64 => "windows-x86_64",
        }
    }
}

/// Returns the registry snapshot version embedded in the crate.
#[must_use]
pub fn registry_version() -> &'static str {
    REGISTRY_VERSION
}

/// Returns the generated registry agent-server definitions.
#[must_use]
pub fn agent_servers() -> Vec<RegistryAgentServer> {
    registry_agent_servers()
}

/// Resolves an official ACP registry id into a generated agent-server
/// definition.
#[must_use]
pub fn agent_server(id: &str) -> Option<RegistryAgentServer> {
    registry_agent_server(id)
}

/// Resolves an official ACP registry id and returns a typed lookup error on
/// failure.
///
/// # Errors
///
/// Returns [`RegistryError::UnknownAgentServer`] when the generated registry
/// snapshot does not contain the requested id.
pub fn require_agent_server(id: &str) -> Result<RegistryAgentServer, RegistryError> {
    agent_server(id).ok_or_else(|| RegistryError::UnknownAgentServer { id: id.to_owned() })
}

/// Resolves the current host into a known ACP registry target.
///
/// # Errors
///
/// Returns [`RegistryError::UnsupportedHostPlatform`] when the build host does
/// not map to a supported registry target in v0.
pub fn host_platform() -> Result<HostPlatform, RegistryError> {
    HostPlatform::from_target(env::consts::OS, env::consts::ARCH)
}

/// Resolves the registry binary target for the current host when the agent
/// server publishes binaries.
///
/// Package-backed agent servers return `Ok(None)`.
///
/// # Errors
///
/// Returns [`RegistryError::UnsupportedHostPlatform`] when the current host is
/// not mapped in v0, or [`RegistryError::MissingBinaryTarget`] when the agent
/// server publishes binaries but not for the current host target.
pub fn host_binary_target(
    agent_server: &RegistryAgentServer,
) -> Result<Option<&RegistryBinaryTarget>, RegistryError> {
    binary_target_for(agent_server, host_platform()?)
}

/// Resolves the registry binary target for a specific host platform when the
/// agent server publishes binaries.
///
/// Package-backed agent servers return `Ok(None)`.
///
/// # Errors
///
/// Returns [`RegistryError::MissingBinaryTarget`] when the agent server
/// publishes binaries but not for the requested host target.
pub fn binary_target_for(
    agent_server: &RegistryAgentServer,
    platform: HostPlatform,
) -> Result<Option<&RegistryBinaryTarget>, RegistryError> {
    let binary_targets = agent_server.distribution().binary_targets();
    if binary_targets.is_empty() {
        return Ok(None);
    }

    binary_targets
        .iter()
        .find(|target| target.target() == platform.registry_target())
        .map_or_else(
            || {
                Err(RegistryError::MissingBinaryTarget {
                    id: agent_server.id().to_owned(),
                    target: platform.registry_target().to_owned(),
                })
            },
            |target| Ok(Some(target)),
        )
}
