use std::{cell::Cell, error::Error as StdError};

use agent_client_protocol::{self as acp, Client as _};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

pub type PendingSessionUpdate = (acp::SessionNotification, oneshot::Sender<()>);

#[derive(Debug)]
pub struct FixtureAgent {
    session_update_tx: mpsc::UnboundedSender<PendingSessionUpdate>,
    next_session_id: Cell<u64>,
}

impl FixtureAgent {
    pub fn new(session_update_tx: mpsc::UnboundedSender<PendingSessionUpdate>) -> Self {
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
                acp::Implementation::new("acpx-fixture-agent", "0.0.1").title("acpx Fixture Agent"),
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

    async fn prompt(&self, args: acp::PromptRequest) -> Result<acp::PromptResponse, acp::Error> {
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

#[allow(dead_code)]
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

    Ok(())
}
