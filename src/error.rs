use std::io;

use thiserror::Error;

/// The crate-wide result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors surfaced by `acpx`.
#[derive(Debug, Error)]
pub enum Error {
    /// The requested agent launch mode is not supported by this crate version.
    #[error(transparent)]
    UnsupportedLaunch(#[from] UnsupportedLaunch),

    /// Spawning the agent subprocess failed.
    #[error("failed to spawn agent process")]
    SpawnProcess {
        /// The underlying I/O error from process creation.
        #[source]
        source: io::Error,
    },

    /// The spawned agent process did not expose a piped stdin handle.
    #[error("agent process stdin was not captured")]
    MissingChildStdin,

    /// The spawned agent process did not expose a piped stdout handle.
    #[error("agent process stdout was not captured")]
    MissingChildStdout,

    /// ACP request, response, or transport handling failed.
    #[error(transparent)]
    Protocol(#[from] agent_client_protocol::Error),

    /// The connection has already been closed.
    #[error("connection is closed")]
    Closed,

    /// Waiting for the agent process to exit failed.
    #[error("failed to wait for agent process exit")]
    WaitForProcess {
        /// The underlying I/O error from waiting on the child process.
        #[source]
        source: io::Error,
    },

    /// Terminating the agent process failed.
    #[error("failed to terminate agent process")]
    KillProcess {
        /// The underlying I/O error from killing the child process.
        #[source]
        source: io::Error,
    },

    /// The ACP I/O task ended unexpectedly.
    #[error("acp connection io task ended unexpectedly")]
    IoTaskTerminated,
}

/// Launch strategies that are intentionally unsupported in `acpx` v0.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnsupportedLaunch {
    /// The official registry describes a downloadable binary distribution.
    #[error("binary registry distributions are not supported in v0")]
    BinaryDistribution,

    /// The registry or manual server definition uses a launcher that is not
    /// connectable by this crate version.
    #[error("launch command `{command}` is not supported in v0")]
    UnsupportedCommand {
        /// The unsupported command or launcher identifier.
        command: String,
    },
}

#[cfg(test)]
mod tests {
    use super::UnsupportedLaunch;

    #[test]
    fn unsupported_launch_messages_are_stable() {
        assert_eq!(
            UnsupportedLaunch::BinaryDistribution.to_string(),
            "binary registry distributions are not supported in v0"
        );
        assert_eq!(
            UnsupportedLaunch::UnsupportedCommand {
                command: "brew".into()
            }
            .to_string(),
            "launch command `brew` is not supported in v0"
        );
    }
}
