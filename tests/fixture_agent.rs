mod support;

use std::path::PathBuf;

use agent_client_protocol::{self as acp, Agent as _};

use self::support::fixture_agent::FixtureAgent;

#[tokio::test(flavor = "current_thread")]
async fn fixture_agent_issues_incrementing_session_ids() {
    let (session_update_tx, _session_update_rx) = tokio::sync::mpsc::unbounded_channel();
    let agent = FixtureAgent::new(session_update_tx);

    let first = agent
        .new_session(acp::NewSessionRequest::new(PathBuf::from(
            "/tmp/project-one",
        )))
        .await
        .expect("first session should succeed");
    let second = agent
        .new_session(acp::NewSessionRequest::new(PathBuf::from(
            "/tmp/project-two",
        )))
        .await
        .expect("second session should succeed");

    assert_eq!(first.session_id, acp::SessionId::new("fixture-session-0"));
    assert_eq!(second.session_id, acp::SessionId::new("fixture-session-1"));
}

#[tokio::test(flavor = "current_thread")]
async fn fixture_agent_streams_prompt_blocks_back_as_updates() {
    let (session_update_tx, mut session_update_rx) = tokio::sync::mpsc::unbounded_channel();
    let agent = FixtureAgent::new(session_update_tx);
    let session_id = acp::SessionId::new("fixture-session-0");
    let prompt = vec!["hello".into(), "world".into()];

    let (response, updates) = tokio::join!(
        agent.prompt(acp::PromptRequest::new(session_id.clone(), prompt)),
        async move {
            let mut seen = Vec::new();
            for _ in 0..2 {
                let (notification, ack) = session_update_rx
                    .recv()
                    .await
                    .expect("expected a streamed session update");
                seen.push(notification);
                ack.send(()).expect("ack should be accepted");
            }
            seen
        }
    );

    let response = response.expect("prompt should succeed");
    assert_eq!(response.stop_reason, acp::StopReason::EndTurn);
    assert_eq!(updates.len(), 2);

    for (index, notification) in updates.into_iter().enumerate() {
        assert_eq!(notification.session_id, session_id);
        let acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) =
            notification.update
        else {
            panic!("expected an agent message chunk");
        };
        let acp::ContentBlock::Text(text) = content else {
            panic!("expected text content");
        };
        assert_eq!(text.text, ["hello", "world"][index]);
    }
}
