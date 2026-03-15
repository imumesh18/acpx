mod support;

use std::path::PathBuf;

use acpx::{AgentMetadata, AgentServer, CommandAgentServer, CommandSpec, Error, RuntimeContext};
use agent_client_protocol::{self as acp};

#[test]
fn fixture_agent_child_entrypoint() {
    if std::env::var_os("ACPX_FIXTURE_AGENT_CHILD").is_some() {
        support::fixture_agent::run_stdio_main().expect("fixture agent child should run");
        std::process::exit(0);
    }
}

#[test]
fn command_agent_server_exposes_metadata() {
    let server = fixture_server();

    assert_eq!(server.id(), "fixture-agent");
    assert_eq!(server.name(), "Fixture Agent");
    assert_eq!(server.description(), "Manual command-backed ACP fixture");
    assert_eq!(server.version(), "0.0.1");
    assert_eq!(server.icon(), Some("fixture.svg"));
    assert_eq!(
        server.command().program(),
        current_test_binary().to_string_lossy()
    );
}

#[test]
fn command_agent_server_connects_and_closes_fixture_agent() {
    run_local_test(async {
        let runtime = runtime_context();
        let server = fixture_server();
        let connection = server
            .connect(&runtime)
            .await
            .expect("manual command-backed server should connect");
        let initialize = connection
            .initialize(initialize_request())
            .await
            .expect("initialize should succeed");
        let session = connection
            .new_session(acp::NewSessionRequest::new(PathBuf::from(
                "/tmp/acpx-agent-server",
            )))
            .await
            .expect("new_session should succeed");

        assert_eq!(initialize.protocol_version, acp::ProtocolVersion::V1);
        assert_eq!(session.session_id, acp::SessionId::new("fixture-session-0"));

        server
            .close(&connection)
            .await
            .expect("server close should delegate to connection");
        assert!(matches!(
            connection.initialize(initialize_request()).await,
            Err(Error::Closed)
        ));
    });
}

fn fixture_server() -> CommandAgentServer {
    CommandAgentServer::new(
        AgentMetadata::new("fixture-agent", "Fixture Agent", "0.0.1")
            .description("Manual command-backed ACP fixture")
            .icon("fixture.svg"),
        CommandSpec::new(current_test_binary().to_string_lossy())
            .arg("--exact")
            .arg("fixture_agent_child_entrypoint")
            .arg("--nocapture")
            .env("ACPX_FIXTURE_AGENT_CHILD", "1"),
    )
}

fn current_test_binary() -> PathBuf {
    std::env::current_exe().expect("test binary path should exist")
}

fn run_local_test(test: impl std::future::Future<Output = ()>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build");
    let local_set = tokio::task::LocalSet::new();

    runtime.block_on(local_set.run_until(test));
}

fn runtime_context() -> RuntimeContext {
    RuntimeContext::new(|task| {
        tokio::task::spawn_local(task);
    })
}

fn initialize_request() -> acp::InitializeRequest {
    acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
        acp::Implementation::new("acpx-test-client", "0.0.1").title("acpx Test Client"),
    )
}
