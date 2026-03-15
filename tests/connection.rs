mod support;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use acpx::{Connection, Error, RuntimeContext};
use agent_client_protocol::{self as acp};
use async_process::{Command, Stdio};
use futures::StreamExt as _;

#[test]
fn fixture_agent_child_entrypoint() {
    if std::env::var_os("ACPX_FIXTURE_AGENT_CHILD").is_some() {
        support::fixture_agent::run_stdio_main().expect("fixture agent child should run");
        std::process::exit(0);
    }
}

#[test]
fn connection_forwards_requests_and_captures_session_updates() {
    run_local_test(async {
        let mut command = fixture_agent_command();
        let runtime = runtime_context();
        let connection =
            Connection::spawn(&mut command, &runtime).expect("connection should spawn");
        let initialize = connection
            .initialize(initialize_request())
            .await
            .expect("initialize should succeed");
        let mut session_updates = connection.subscribe_session_updates();
        let session = connection
            .new_session(acp::NewSessionRequest::new(PathBuf::from(
                "/tmp/acpx-connection",
            )))
            .await
            .expect("new_session should succeed");
        let prompt = connection
            .prompt(acp::PromptRequest::new(
                session.session_id.clone(),
                vec!["hello".into(), "world".into()],
            ))
            .await
            .expect("prompt should succeed");
        let first_update = session_updates
            .next()
            .await
            .expect("expected a first session update");
        let second_update = session_updates
            .next()
            .await
            .expect("expected a second session update");

        assert_eq!(initialize.protocol_version, acp::ProtocolVersion::V1);
        assert_eq!(
            initialize
                .agent_info
                .expect("fixture agent should report metadata")
                .name,
            "acpx-fixture-agent"
        );
        assert_eq!(prompt.stop_reason, acp::StopReason::EndTurn);
        assert_eq!(session.session_id, acp::SessionId::new("fixture-session-0"));
        assert_session_update(first_update, &session.session_id, "hello");
        assert_session_update(second_update, &session.session_id, "world");

        connection.close().await.expect("close should succeed");
    });
}

#[test]
fn connection_close_is_idempotent_and_marks_the_connection_closed() {
    run_local_test(async {
        let mut command = fixture_agent_command();
        let runtime = runtime_context();
        let connection =
            Connection::spawn(&mut command, &runtime).expect("connection should spawn");
        let pid = connection
            .process_id()
            .expect("spawned connection should expose a process id");

        connection
            .close()
            .await
            .expect("first close should succeed");
        connection
            .close()
            .await
            .expect("second close should also succeed");

        assert!(matches!(
            connection.initialize(initialize_request()).await,
            Err(Error::Closed)
        ));
        assert!(connection.process_id().is_none());
        wait_for_process_exit(pid);
    });
}

#[test]
fn dropping_connection_terminates_the_child_process() {
    run_local_test(async {
        let pid = {
            let mut command = fixture_agent_command();
            let runtime = runtime_context();
            let connection =
                Connection::spawn(&mut command, &runtime).expect("connection should spawn");
            connection
                .process_id()
                .expect("spawned connection should expose a process id")
        };

        wait_for_process_exit(pid);
    });
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

fn fixture_agent_command() -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test binary path should exist"));
    command.arg("--exact");
    command.arg("fixture_agent_child_entrypoint");
    command.arg("--nocapture");
    command.env("ACPX_FIXTURE_AGENT_CHILD", "1");
    command.stderr(Stdio::null());
    command
}

fn initialize_request() -> acp::InitializeRequest {
    acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_info(
        acp::Implementation::new("acpx-test-client", "0.0.1").title("acpx Test Client"),
    )
}

fn assert_session_update(
    notification: acp::SessionNotification,
    session_id: &acp::SessionId,
    expected_text: &str,
) {
    assert_eq!(&notification.session_id, session_id);
    let acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) =
        notification.update
    else {
        panic!("expected an agent message chunk");
    };
    let acp::ContentBlock::Text(text) = content else {
        panic!("expected text content");
    };
    assert_eq!(text.text, expected_text);
}

fn wait_for_process_exit(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);

    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }

        std::thread::sleep(Duration::from_millis(25));
    }

    panic!("process {pid} did not exit in time");
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .expect("tasklist should run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    !stdout.contains("No tasks are running") && stdout.contains(&format!(",\"{pid}\""))
}
