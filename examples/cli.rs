use std::{
    env, io,
    path::{Path, PathBuf},
};

use acpx::{AgentServer, Error as AcpxError, RuntimeContext, agent_servers};
use agent_client_protocol as acp;
use futures::StreamExt as _;
use thiserror::Error;
use tokio::{sync::oneshot, task::JoinHandle};

const CLIENT_NAME: &str = "acpx-cli";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CLIENT_TITLE: &str = "acpx CLI";
const CLI_AGENT_SERVER_ALIASES: &[(&str, &str)] = &[
    ("amp", "amp-acp"),
    ("claude", "claude-acp"),
    ("codebuddy", "codebuddy-code"),
    ("codex", "codex-acp"),
    ("copilot", "github-copilot-cli"),
    ("corust", "corust-agent"),
    ("crow", "crow-cli"),
    ("droid", "factory-droid"),
    ("mcode", "minion-code"),
    ("pi", "pi-acp"),
    ("qwen", "qwen-code"),
    ("vibe", "mistral-vibe"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct CliRequest {
    agent_ref: String,
    prompt: String,
    cwd: Option<PathBuf>,
    mode: Option<String>,
    permission_mode: Option<String>,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    AgentServers(#[from] agent_servers::Error),

    #[error(transparent)]
    Acpx(#[from] AcpxError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

fn main() {
    match try_main() {
        Ok(()) => {}
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            eprintln!();
            eprintln!("{}", usage());
            std::process::exit(2);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<(), CliError> {
    let request = parse_args(env::args().skip(1))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local_set = tokio::task::LocalSet::new();

    runtime.block_on(local_set.run_until(async move {
        let agent_server = resolve_agent_server(&request.agent_ref)?;
        run_cli_with_server(&agent_server, &request).await
    }))
}

fn usage() -> &'static str {
    "usage: cargo run --example cli -- <agent-id-or-alias> <PROMPT> [--cwd <path>] [--mode <mode-id>] [--permission-mode <mode-id>]"
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliRequest, CliError> {
    let mut args = args.into_iter();
    let mut agent_ref = None;
    let mut prompt = None;
    let mut cwd = None;
    let mut mode = None;
    let mut permission_mode = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err(CliError::Usage("help requested".to_owned())),
            "--cwd" => cwd = Some(next_option_value(&mut args, "--cwd")?.into()),
            "--mode" => mode = Some(next_option_value(&mut args, "--mode")?),
            "--permission-mode" => {
                permission_mode = Some(next_option_value(&mut args, "--permission-mode")?);
            }
            _ if agent_ref.is_none() => agent_ref = Some(arg),
            _ if prompt.is_none() => prompt = Some(arg),
            _ => {
                return Err(CliError::Usage(format!("unexpected argument `{arg}`")));
            }
        }
    }

    let Some(agent_ref) = agent_ref else {
        return Err(CliError::Usage(
            "missing required <agent-id-or-alias>".to_owned(),
        ));
    };
    let Some(prompt) = prompt else {
        return Err(CliError::Usage("missing required <PROMPT>".to_owned()));
    };

    Ok(CliRequest {
        agent_ref,
        prompt,
        cwd,
        mode,
        permission_mode,
    })
}

fn next_option_value(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, CliError> {
    args.next()
        .ok_or_else(|| CliError::Usage(format!("missing value for `{option}`")))
}

fn resolve_agent_server(agent_ref: &str) -> Result<agent_servers::Server, CliError> {
    agent_servers::get(agent_ref)
        .or_else(|| alias_agent_server_id(agent_ref).and_then(agent_servers::get))
        .ok_or_else(|| {
            agent_servers::Error::UnknownServer {
                id: agent_ref.to_owned(),
            }
            .into()
        })
}

fn alias_agent_server_id(agent_ref: &str) -> Option<&'static str> {
    CLI_AGENT_SERVER_ALIASES
        .iter()
        .find_map(|(alias, target)| (*alias == agent_ref).then_some(*target))
}

async fn run_cli_with_server<S>(server: &S, request: &CliRequest) -> Result<(), CliError>
where
    S: AgentServer,
{
    let runtime = RuntimeContext::new(|task| {
        tokio::task::spawn_local(task);
    });
    let connection = server.connect(&runtime).await?;
    let (stop_updates, updates_task) = spawn_session_update_printer(&connection);
    let run_result = run_connected_session(server, &connection, request).await;
    tokio::task::yield_now().await;
    let _ = stop_updates.send(());
    let _ = updates_task.await;
    let close_result = connection.close().await;

    run_result?;
    close_result?;
    Ok(())
}

async fn run_connected_session<S>(
    server: &S,
    connection: &acpx::Connection,
    request: &CliRequest,
) -> Result<(), CliError>
where
    S: AgentServer,
{
    let cwd = resolve_cwd(request.cwd.as_deref())?;
    let initialize = connection
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
                acp::Implementation::new(CLIENT_NAME, CLIENT_VERSION).title(CLIENT_TITLE),
            ),
        )
        .await?;

    if request.agent_ref == server.id() {
        println!("resolved agent: {} ({})", server.name(), server.id());
    } else {
        println!(
            "resolved agent: {} ({}) via `{}`",
            server.name(),
            server.id(),
            request.agent_ref
        );
    }
    println!("initialize: {initialize:#?}");
    println!("auth methods: {:#?}", initialize.auth_methods);

    let session = connection
        .new_session(acp::NewSessionRequest::new(cwd))
        .await?;
    println!("session/new: {session:#?}");

    if let Some(mode) = &request.mode {
        let response = connection
            .set_session_mode(acp::SetSessionModeRequest::new(
                session.session_id.clone(),
                mode.clone(),
            ))
            .await?;
        println!("session/set_mode: {response:#?}");
    }

    if let Some(permission_mode) = &request.permission_mode {
        apply_permission_mode(
            connection,
            &session.session_id,
            session.config_options.as_deref(),
            permission_mode,
        )
        .await?;
    }

    let prompt_response = connection
        .prompt(acp::PromptRequest::new(
            session.session_id.clone(),
            vec![request.prompt.clone().into()],
        ))
        .await?;
    println!("prompt: {prompt_response:#?}");

    Ok(())
}

fn spawn_session_update_printer(
    connection: &acpx::Connection,
) -> (oneshot::Sender<()>, JoinHandle<()>) {
    let mut updates = connection.subscribe_session_updates();
    let (stop_tx, mut stop_rx) = oneshot::channel();
    let handle = tokio::task::spawn_local(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                notification = updates.next() => match notification {
                    Some(notification) => println!("session/update: {notification:#?}"),
                    None => break,
                },
            }
        }
    });

    (stop_tx, handle)
}

async fn apply_permission_mode(
    connection: &acpx::Connection,
    session_id: &acp::SessionId,
    config_options: Option<&[acp::SessionConfigOption]>,
    permission_mode: &str,
) -> Result<(), CliError> {
    let Some(config_id) = find_permission_option_id(config_options) else {
        println!("session/set_config_option: skipped; no advertised permission option");
        return Ok(());
    };

    let response = connection
        .set_session_config_option(acp::SetSessionConfigOptionRequest::new(
            session_id.clone(),
            config_id,
            permission_mode.to_owned(),
        ))
        .await?;
    println!("session/set_config_option: {response:#?}");

    Ok(())
}

fn find_permission_option_id(
    config_options: Option<&[acp::SessionConfigOption]>,
) -> Option<acp::SessionConfigId> {
    config_options.and_then(|options| {
        options.iter().find_map(|option| {
            if !matches!(option.kind, acp::SessionConfigKind::Select(_)) {
                return None;
            }

            let id = option.id.to_string().to_ascii_lowercase();
            let name = option.name.to_ascii_lowercase();
            if id.contains("permission") || name.contains("permission") {
                Some(option.id.clone())
            } else {
                None
            }
        })
    })
}

fn resolve_cwd(cwd: Option<&Path>) -> Result<PathBuf, CliError> {
    cwd.map_or_else(env::current_dir, absolutize_path)
        .map_err(CliError::from)
}

fn absolutize_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::acpx_cli_fixture_agent;

    use super::{CliRequest, parse_args, resolve_agent_server, run_cli_with_server};
    use acpx::{AgentServer, AgentServerMetadata, CommandAgentServer, CommandSpec};

    #[test]
    fn fixture_agent_child_entrypoint() {
        if std::env::var_os("ACPX_FIXTURE_AGENT_CHILD").is_some() {
            acpx_cli_fixture_agent::run_stdio_main().expect("fixture agent child should run");
            std::process::exit(0);
        }
    }

    #[test]
    fn parse_args_supports_v0_options() {
        let parsed = parse_args([
            "codex".to_owned(),
            "hello".to_owned(),
            "--cwd".to_owned(),
            "workspace".to_owned(),
            "--mode".to_owned(),
            "edit".to_owned(),
            "--permission-mode".to_owned(),
            "ask".to_owned(),
        ])
        .expect("arguments should parse");

        assert_eq!(
            parsed,
            CliRequest {
                agent_ref: "codex".to_owned(),
                prompt: "hello".to_owned(),
                cwd: Some(PathBuf::from("workspace")),
                mode: Some("edit".to_owned()),
                permission_mode: Some("ask".to_owned()),
            }
        );
    }

    #[test]
    fn agent_server_resolution_supports_cli_aliases() {
        let codex = resolve_agent_server("codex").expect("codex alias should resolve");
        assert_eq!(codex.id(), "codex-acp");

        let droid = resolve_agent_server("droid").expect("droid alias should resolve");
        assert_eq!(droid.id(), "factory-droid");

        let vibe = resolve_agent_server("vibe").expect("vibe alias should resolve");
        assert_eq!(vibe.id(), "mistral-vibe");

        let direct = resolve_agent_server("claude-acp").expect("raw registry id should resolve");
        assert_eq!(direct.id(), "claude-acp");
    }

    #[test]
    fn manual_command_server_smoke_runs_single_shot_flow() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime should build");
        let local_set = tokio::task::LocalSet::new();

        runtime.block_on(local_set.run_until(async move {
            let server = CommandAgentServer::new(
                AgentServerMetadata::new("fixture-agent", "Fixture Agent", "0.0.1"),
                fixture_agent_command(),
            );
            let request = CliRequest {
                agent_ref: "fixture-agent".to_owned(),
                prompt: "hello from cli".to_owned(),
                cwd: Some(std::env::current_dir().expect("current directory should be available")),
                mode: Some("chat".to_owned()),
                permission_mode: Some("auto-edit".to_owned()),
            };

            run_cli_with_server(&server, &request)
                .await
                .expect("cli flow should succeed");
        }));
    }

    fn fixture_agent_command() -> CommandSpec {
        CommandSpec::new(
            std::env::current_exe()
                .expect("test binary path should exist")
                .to_string_lossy()
                .into_owned(),
        )
        .arg("--exact")
        .arg("tests::fixture_agent_child_entrypoint")
        .arg("--nocapture")
        .env("ACPX_FIXTURE_AGENT_CHILD", "1")
    }
}

#[cfg(test)]
mod acpx_cli_fixture_agent {
    use std::{
        cell::Cell,
        error::Error as StdError,
        io::{self, Write as _},
    };

    use agent_client_protocol::{self as acp, Client as _};
    use tokio::sync::{mpsc, oneshot};
    use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

    type PendingSessionUpdate = (acp::SessionNotification, oneshot::Sender<()>);

    #[derive(Debug)]
    struct FixtureAgent {
        session_update_tx: mpsc::UnboundedSender<PendingSessionUpdate>,
        next_session_id: Cell<u64>,
    }

    impl FixtureAgent {
        fn new(session_update_tx: mpsc::UnboundedSender<PendingSessionUpdate>) -> Self {
            Self {
                session_update_tx,
                next_session_id: Cell::new(0),
            }
        }
    }

    #[async_trait::async_trait(?Send)]
    impl acp::Agent for FixtureAgent {
        async fn initialize(
            &self,
            args: acp::InitializeRequest,
        ) -> Result<acp::InitializeResponse, acp::Error> {
            Ok(
                acp::InitializeResponse::new(args.protocol_version).agent_info(
                    acp::Implementation::new("acpx-cli-fixture-agent", "0.0.1")
                        .title("acpx CLI Fixture Agent"),
                ),
            )
        }

        async fn authenticate(
            &self,
            _args: acp::AuthenticateRequest,
        ) -> Result<acp::AuthenticateResponse, acp::Error> {
            Ok(acp::AuthenticateResponse::default())
        }

        async fn new_session(
            &self,
            _args: acp::NewSessionRequest,
        ) -> Result<acp::NewSessionResponse, acp::Error> {
            let session_number = self.next_session_id.get();
            self.next_session_id.set(session_number + 1);

            Ok(acp::NewSessionResponse::new(acp::SessionId::new(format!(
                "fixture-session-{session_number}"
            )))
            .config_options(vec![acp::SessionConfigOption::select(
                "permission-mode",
                "Permission Mode",
                "ask",
                vec![
                    acp::SessionConfigSelectOption::new("ask", "Ask"),
                    acp::SessionConfigSelectOption::new("auto-edit", "Auto Edit"),
                ],
            )]))
        }

        async fn load_session(
            &self,
            _args: acp::LoadSessionRequest,
        ) -> Result<acp::LoadSessionResponse, acp::Error> {
            Ok(acp::LoadSessionResponse::new())
        }

        async fn set_session_mode(
            &self,
            _args: acp::SetSessionModeRequest,
        ) -> Result<acp::SetSessionModeResponse, acp::Error> {
            Ok(acp::SetSessionModeResponse::new())
        }

        async fn prompt(
            &self,
            args: acp::PromptRequest,
        ) -> Result<acp::PromptResponse, acp::Error> {
            for content in args.prompt {
                let (tx, rx) = oneshot::channel();
                self.session_update_tx
                    .send((
                        acp::SessionNotification::new(
                            args.session_id.clone(),
                            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(content)),
                        ),
                        tx,
                    ))
                    .map_err(|_| acp::Error::internal_error())?;
                rx.await.map_err(|_| acp::Error::internal_error())?;
            }

            Ok(acp::PromptResponse::new(acp::StopReason::EndTurn))
        }

        async fn cancel(&self, _args: acp::CancelNotification) -> Result<(), acp::Error> {
            Ok(())
        }

        async fn set_session_config_option(
            &self,
            args: acp::SetSessionConfigOptionRequest,
        ) -> Result<acp::SetSessionConfigOptionResponse, acp::Error> {
            let option = acp::SessionConfigOption::select(
                args.config_id,
                "Permission Mode",
                args.value,
                vec![
                    acp::SessionConfigSelectOption::new("ask", "Ask"),
                    acp::SessionConfigSelectOption::new("auto-edit", "Auto Edit"),
                ],
            );

            Ok(acp::SetSessionConfigOptionResponse::new(vec![option]))
        }
    }

    pub fn run_stdio_main() -> Result<(), Box<dyn StdError>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let local_set = tokio::task::LocalSet::new();

        runtime.block_on(local_set.run_until(async move {
            let outgoing = tokio::io::stdout().compat_write();
            let incoming = tokio::io::stdin().compat();
            let (session_update_tx, mut session_update_rx) = mpsc::unbounded_channel();
            let (connection, io_task) = acp::AgentSideConnection::new(
                FixtureAgent::new(session_update_tx),
                outgoing,
                incoming,
                |task| {
                    tokio::task::spawn_local(task);
                },
            );

            tokio::task::spawn_local(async move {
                while let Some((notification, ack)) = session_update_rx.recv().await {
                    if connection.session_notification(notification).await.is_err() {
                        break;
                    }
                    let _ = ack.send(());
                }
            });

            io_task.await
        }))?;

        io::stdout().flush()?;

        Ok(())
    }
}
