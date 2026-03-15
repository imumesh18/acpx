use std::{cell::RefCell, io::ErrorKind, rc::Rc};

use agent_client_protocol::{self as acp, Agent as _};
use async_process::{Child, Command, Stdio};
use futures::channel::{mpsc, oneshot};

use crate::{Error, Result, RuntimeContext};

#[derive(Clone, Debug, Default)]
struct SessionUpdateBroadcaster {
    subscribers: Rc<RefCell<Vec<mpsc::UnboundedSender<acp::SessionNotification>>>>,
}

impl SessionUpdateBroadcaster {
    fn subscribe(&self) -> mpsc::UnboundedReceiver<acp::SessionNotification> {
        let (tx, rx) = mpsc::unbounded();
        self.subscribers.borrow_mut().push(tx);
        rx
    }

    fn publish(&self, notification: &acp::SessionNotification) {
        let mut subscribers = self.subscribers.borrow_mut();
        subscribers.retain(|subscriber| subscriber.unbounded_send(notification.clone()).is_ok());
    }
}

#[derive(Clone, Debug)]
struct DefaultConnectionClient {
    session_updates: SessionUpdateBroadcaster,
}

impl DefaultConnectionClient {
    fn new(session_updates: SessionUpdateBroadcaster) -> Self {
        Self { session_updates }
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for DefaultConnectionClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> acp::Result<()> {
        self.session_updates.publish(&args);
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Err(acp::Error::method_not_found())
    }
}

struct ConnectionState {
    connection: Option<Rc<acp::ClientSideConnection>>,
    child: Option<Child>,
    io_task: Option<oneshot::Receiver<Result<()>>>,
}

/// A connected ACP client bound to a local subprocess.
pub struct Connection {
    session_updates: SessionUpdateBroadcaster,
    state: Rc<RefCell<ConnectionState>>,
}

impl Connection {
    /// Launches a local ACP agent process and wires its stdio into the ACP SDK.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be spawned, if its stdio pipes
    /// are unavailable, or if ACP setup fails.
    pub fn spawn(command: &mut Command, runtime: &RuntimeContext) -> Result<Self> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.kill_on_drop(true);

        let mut child = command
            .spawn()
            .map_err(|source| Error::SpawnProcess { source })?;
        let outgoing = child.stdin.take().ok_or(Error::MissingChildStdin)?;
        let incoming = child.stdout.take().ok_or(Error::MissingChildStdout)?;
        let session_updates = SessionUpdateBroadcaster::default();
        let client = DefaultConnectionClient::new(session_updates.clone());
        let runtime_for_sdk = runtime.clone();
        let (connection, io_task) =
            acp::ClientSideConnection::new(client, outgoing, incoming, move |task| {
                runtime_for_sdk.spawn_local(task);
            });
        let connection = Rc::new(connection);
        let (io_task_tx, io_task_rx) = oneshot::channel();

        runtime.spawn(async move {
            let _ = io_task_tx.send(io_task.await.map_err(Error::from));
        });

        Ok(Self {
            session_updates,
            state: Rc::new(RefCell::new(ConnectionState {
                connection: Some(connection),
                child: Some(child),
                io_task: Some(io_task_rx),
            })),
        })
    }

    /// Returns the spawned process ID while the subprocess is still owned.
    #[must_use]
    pub fn process_id(&self) -> Option<u32> {
        self.state.borrow().child.as_ref().map(Child::id)
    }

    /// Subscribes to raw JSON-RPC stream traffic from the underlying ACP SDK.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection has already been closed.
    pub fn subscribe(&self) -> Result<acp::StreamReceiver> {
        Ok(self.connection()?.subscribe())
    }

    /// Subscribes to captured `session/update` notifications from the agent.
    #[must_use]
    pub fn subscribe_session_updates(&self) -> mpsc::UnboundedReceiver<acp::SessionNotification> {
        self.session_updates.subscribe()
    }

    /// Closes the ACP connection and terminates the child process.
    ///
    /// # Errors
    ///
    /// Returns an error if terminating or waiting for the child process fails.
    pub async fn close(&self) -> Result<()> {
        let (connection, mut child, io_task) = {
            let mut state = self.state.borrow_mut();
            let Some(connection) = state.connection.take() else {
                return Ok(());
            };

            (connection, state.child.take(), state.io_task.take())
        };

        drop(connection);

        if let Some(child) = child.as_mut() {
            match child.kill() {
                Ok(()) => {}
                Err(source) if source.kind() == ErrorKind::InvalidInput => {}
                Err(source) => return Err(Error::KillProcess { source }),
            }
        }

        if let Some(mut child) = child {
            child
                .status()
                .await
                .map_err(|source| Error::WaitForProcess { source })?;
        }

        if let Some(io_task) = io_task {
            let _ = io_task.await;
        }

        Ok(())
    }

    /// Forwards the ACP initialize request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn initialize(
        &self,
        args: acp::InitializeRequest,
    ) -> Result<acp::InitializeResponse> {
        let connection = self.connection()?;
        connection.initialize(args).await.map_err(Error::from)
    }

    /// Forwards the ACP authenticate request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn authenticate(
        &self,
        args: acp::AuthenticateRequest,
    ) -> Result<acp::AuthenticateResponse> {
        let connection = self.connection()?;
        connection.authenticate(args).await.map_err(Error::from)
    }

    /// Forwards the ACP session creation request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn new_session(
        &self,
        args: acp::NewSessionRequest,
    ) -> Result<acp::NewSessionResponse> {
        let connection = self.connection()?;
        connection.new_session(args).await.map_err(Error::from)
    }

    /// Forwards the ACP session loading request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn load_session(
        &self,
        args: acp::LoadSessionRequest,
    ) -> Result<acp::LoadSessionResponse> {
        let connection = self.connection()?;
        connection.load_session(args).await.map_err(Error::from)
    }

    /// Forwards the ACP session mode request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn set_session_mode(
        &self,
        args: acp::SetSessionModeRequest,
    ) -> Result<acp::SetSessionModeResponse> {
        let connection = self.connection()?;
        connection.set_session_mode(args).await.map_err(Error::from)
    }

    /// Forwards the ACP prompt request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn prompt(&self, args: acp::PromptRequest) -> Result<acp::PromptResponse> {
        let connection = self.connection()?;
        connection.prompt(args).await.map_err(Error::from)
    }

    /// Forwards the ACP cancel notification.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn cancel(&self, args: acp::CancelNotification) -> Result<()> {
        let connection = self.connection()?;
        connection.cancel(args).await.map_err(Error::from)
    }

    /// Forwards the ACP list sessions request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn list_sessions(
        &self,
        args: acp::ListSessionsRequest,
    ) -> Result<acp::ListSessionsResponse> {
        let connection = self.connection()?;
        connection.list_sessions(args).await.map_err(Error::from)
    }

    /// Forwards the ACP session config option request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn set_session_config_option(
        &self,
        args: acp::SetSessionConfigOptionRequest,
    ) -> Result<acp::SetSessionConfigOptionResponse> {
        let connection = self.connection()?;
        connection
            .set_session_config_option(args)
            .await
            .map_err(Error::from)
    }

    /// Forwards an ACP extension request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn ext_method(&self, args: acp::ExtRequest) -> Result<acp::ExtResponse> {
        let connection = self.connection()?;
        connection.ext_method(args).await.map_err(Error::from)
    }

    /// Forwards an ACP extension notification.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Closed`] if the connection is closed, or wraps the ACP
    /// error returned by the underlying SDK.
    pub async fn ext_notification(&self, args: acp::ExtNotification) -> Result<()> {
        let connection = self.connection()?;
        connection.ext_notification(args).await.map_err(Error::from)
    }

    fn connection(&self) -> Result<Rc<acp::ClientSideConnection>> {
        self.state.borrow().connection.clone().ok_or(Error::Closed)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let (connection, child, io_task) = {
            let mut state = self.state.borrow_mut();
            (
                state.connection.take(),
                state.child.take(),
                state.io_task.take(),
            )
        };

        drop(connection);
        drop(io_task);
        drop(child);
    }
}
