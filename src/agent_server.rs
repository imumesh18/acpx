use std::{io::ErrorKind, path::PathBuf, process::Command as StdCommand};

use async_process::Command;

use crate::{Connection, Error, Result, RuntimeContext, Task};

/// Descriptive metadata for a launchable ACP agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMetadata {
    id: String,
    name: String,
    description: String,
    version: String,
    icon: Option<String>,
}

impl AgentMetadata {
    /// Creates the minimum required metadata for an agent definition.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            version: version.into(),
            icon: None,
        }
    }

    /// Sets the human-readable agent description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the optional agent icon.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Returns the stable agent identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description.
    #[must_use]
    pub fn description_text(&self) -> &str {
        &self.description
    }

    /// Returns the version string.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the optional icon reference.
    #[must_use]
    pub fn icon_ref(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// A reusable subprocess launch specification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

impl CommandSpec {
    /// Creates a launch specification from a program name or absolute path.
    #[must_use]
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
        }
    }

    /// Appends a single argument.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Appends multiple arguments in order.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Adds an environment variable for the launched process.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Sets the working directory for the launched process.
    #[must_use]
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Returns the configured executable name or path.
    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the configured argument list.
    #[must_use]
    pub fn args_ref(&self) -> &[String] {
        &self.args
    }

    /// Returns the configured environment overrides.
    #[must_use]
    pub fn env_ref(&self) -> &[(String, String)] {
        &self.env
    }

    /// Returns the configured working directory.
    #[must_use]
    pub fn cwd_ref(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    fn to_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command
    }
}

/// A handwritten contract for launchable ACP agent definitions.
pub trait AgentServer {
    /// Returns the durable metadata for this server.
    fn metadata(&self) -> &AgentMetadata;

    /// Launches the server and returns a connected ACP client handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the subprocess cannot be started or if ACP wiring
    /// fails during connection setup.
    fn connect<'a>(&'a self, runtime: &'a RuntimeContext) -> Task<'a, Result<Connection>>;

    /// Closes a live connection previously returned by [`Self::connect`].
    ///
    /// # Errors
    ///
    /// Returns any shutdown error from the underlying connection handle.
    fn close<'a>(&'a self, connection: &'a Connection) -> Task<'a, Result<()>> {
        Box::pin(async move { connection.close().await })
    }

    /// Returns the stable agent identifier.
    #[must_use]
    fn id(&self) -> &str {
        self.metadata().id()
    }

    /// Returns the display name.
    #[must_use]
    fn name(&self) -> &str {
        self.metadata().name()
    }

    /// Returns the human-readable description.
    #[must_use]
    fn description(&self) -> &str {
        self.metadata().description_text()
    }

    /// Returns the version string.
    #[must_use]
    fn version(&self) -> &str {
        self.metadata().version()
    }

    /// Returns the optional icon reference.
    #[must_use]
    fn icon(&self) -> Option<&str> {
        self.metadata().icon_ref()
    }
}

/// A handwritten ACP server definition backed by a fixed subprocess command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandAgentServer {
    metadata: AgentMetadata,
    command: CommandSpec,
}

impl CommandAgentServer {
    /// Creates a command-backed agent server definition.
    #[must_use]
    pub fn new(metadata: AgentMetadata, command: CommandSpec) -> Self {
        Self { metadata, command }
    }

    /// Returns the launch specification.
    #[must_use]
    pub fn command(&self) -> &CommandSpec {
        &self.command
    }
}

impl AgentServer for CommandAgentServer {
    fn metadata(&self) -> &AgentMetadata {
        &self.metadata
    }

    fn connect<'a>(&'a self, runtime: &'a RuntimeContext) -> Task<'a, Result<Connection>> {
        Box::pin(async move {
            let mut command = self.command.to_command();
            match Connection::spawn(&mut command, runtime) {
                Err(Error::SpawnProcess { source }) if source.kind() == ErrorKind::NotFound => {
                    Err(Error::MissingLauncher {
                        launcher: self.command.program().to_owned(),
                        source,
                    })
                }
                result => result,
            }
        })
    }
}

impl From<StdCommand> for CommandSpec {
    fn from(command: StdCommand) -> Self {
        let program = command.get_program().to_string_lossy().into_owned();
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let cwd = command.get_current_dir().map(PathBuf::from);
        let env = command
            .get_envs()
            .filter_map(|(key, value)| {
                value.map(|value| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
            })
            .collect();

        Self {
            program,
            args,
            env,
            cwd,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use futures::executor::block_on;

    use super::{AgentMetadata, AgentServer, CommandAgentServer, CommandSpec};
    use crate::{Error, RuntimeContext};

    #[test]
    fn metadata_builder_sets_optional_fields() {
        let metadata = AgentMetadata::new("fixture", "Fixture Agent", "0.0.1")
            .description("Manual test agent")
            .icon("fixture.svg");

        assert_eq!(metadata.id(), "fixture");
        assert_eq!(metadata.name(), "Fixture Agent");
        assert_eq!(metadata.description_text(), "Manual test agent");
        assert_eq!(metadata.version(), "0.0.1");
        assert_eq!(metadata.icon_ref(), Some("fixture.svg"));
    }

    #[test]
    fn command_spec_builder_preserves_launch_details() {
        let spec = CommandSpec::new("uvx")
            .arg("--from")
            .arg("package")
            .env("ACP_MODE", "test")
            .cwd("/tmp/project");

        assert_eq!(spec.program(), "uvx");
        assert_eq!(spec.args_ref(), ["--from", "package"]);
        assert_eq!(spec.env_ref(), [("ACP_MODE".to_owned(), "test".to_owned())]);
        assert_eq!(spec.cwd_ref(), Some(&PathBuf::from("/tmp/project")));
    }

    #[test]
    fn command_agent_server_surfaces_missing_launchers() {
        let runtime = RuntimeContext::new(|task| {
            block_on(task);
        });
        let server = CommandAgentServer::new(
            AgentMetadata::new("missing", "Missing Launcher", "0.0.1"),
            CommandSpec::new("acpx-launcher-that-should-not-exist"),
        );

        let error = block_on(server.connect(&runtime)).err();

        assert!(matches!(
            error,
            Some(Error::MissingLauncher { launcher, .. })
                if launcher == "acpx-launcher-that-should-not-exist"
        ));
    }
}
