use std::env;

use thiserror::Error;

use crate::agent_server::AgentServer;
use crate::agent_servers::{
    GeneratedAgentServer, GeneratedBinaryTarget, REGISTRY_VERSION, generated_agent_server,
    generated_agent_servers,
};

const COPILOT_ALIAS: &str = "copilot";
const COPILOT_CANONICAL_ID: &str = "github-copilot-cli";
const HIDDEN_CURATED_NAMES: &[&str] = &["github-copilot"];

/// Typed failures from the curated registry mapping layer.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum RegistryError {
    /// The curated registry does not resolve the requested name.
    #[error("curated agent `{id}` was not found")]
    UnknownAgent {
        /// The unresolved agent name, alias, or identifier.
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
    #[error("agent `{id}` does not publish a binary for host platform `{target}`")]
    MissingBinaryTarget {
        /// The registry id for the agent.
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

/// Returns the canonical registry id for a curated alias.
#[must_use]
pub fn alias_target(alias: &str) -> Option<&'static str> {
    match alias {
        COPILOT_ALIAS => Some(COPILOT_CANONICAL_ID),
        _ => None,
    }
}

/// Returns the curated agent catalog used by the public registry layer.
#[must_use]
pub fn curated_agent_servers() -> Vec<GeneratedAgentServer> {
    generated_agent_servers()
}

/// Resolves a curated registry id or alias into a generated agent server.
#[must_use]
pub fn lookup(id_or_alias: &str) -> Option<GeneratedAgentServer> {
    if HIDDEN_CURATED_NAMES.contains(&id_or_alias) {
        return None;
    }

    alias_target(id_or_alias)
        .and_then(generated_agent_server)
        .or_else(|| generated_agent_server(id_or_alias))
}

/// Resolves a curated registry id or alias and returns a typed lookup error on
/// failure.
///
/// # Errors
///
/// Returns [`RegistryError::UnknownAgent`] when the curated registry does not
/// resolve the requested name.
pub fn require(id_or_alias: &str) -> Result<GeneratedAgentServer, RegistryError> {
    lookup(id_or_alias).ok_or_else(|| RegistryError::UnknownAgent {
        id: id_or_alias.to_owned(),
    })
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
/// publishes binaries.
///
/// Package-backed agents return `Ok(None)`.
///
/// # Errors
///
/// Returns [`RegistryError::UnsupportedHostPlatform`] when the current host is
/// not mapped in v0, or [`RegistryError::MissingBinaryTarget`] when the agent
/// publishes binaries but not for the current host target.
pub fn host_binary_target(
    agent: &GeneratedAgentServer,
) -> Result<Option<&GeneratedBinaryTarget>, RegistryError> {
    binary_target_for(agent, host_platform()?)
}

/// Resolves the registry binary target for a specific host platform when the
/// agent publishes binaries.
///
/// Package-backed agents return `Ok(None)`.
///
/// # Errors
///
/// Returns [`RegistryError::MissingBinaryTarget`] when the agent publishes
/// binaries but not for the requested host target.
pub fn binary_target_for(
    agent: &GeneratedAgentServer,
    platform: HostPlatform,
) -> Result<Option<&GeneratedBinaryTarget>, RegistryError> {
    let binaries = agent.distribution().binary_targets();
    if binaries.is_empty() {
        return Ok(None);
    }

    binaries
        .iter()
        .find(|target| target.target() == platform.registry_target())
        .map_or_else(
            || {
                Err(RegistryError::MissingBinaryTarget {
                    id: agent.metadata().id().to_owned(),
                    target: platform.registry_target().to_owned(),
                })
            },
            |target| Ok(Some(target)),
        )
}
