use crate::{
    Backend, Client, Connection, ConnectionOptions, Error, Frames, NoHandler, PresentationError,
    UiOptions, connect,
};

use nvim_rpc_plugin::{ConnectionError, Handler};

use tokio::process::{Child, ChildStdin, Command};

use std::process::{ExitStatus, Stdio};

/// An owned Neovim child and its shared RPC/UI connection.
///
/// Dropping this value stops RPC and kills the child without saving buffers.
/// Use [`Self::shutdown()`] to kill and reap it explicitly. An external process
/// connected through [`connect()`] is owned by the application instead.
pub struct Embedded {
    connection: Connection,
    child: Child,
}

impl Embedded {
    /// Borrows UI operations and the shared RPC client via [`Client::nvim()`].
    pub fn client(&self) -> &Client<ChildStdin> {
        &self.connection.client
    }
    /// Borrows the pending-frame receiver for custom delivery.
    pub fn frames(&mut self) -> &mut Frames {
        &mut self.connection.frames
    }
    /// Presents the next frame on the backend's owning thread.
    ///
    /// Has the same cancellation and error contracts as [`Connection::present_next()`].
    pub async fn present_next<B: Backend + ?Sized>(
        &mut self,
        backend: &mut B,
    ) -> Result<bool, PresentationError<B::Error>> {
        self.connection.present_next(backend).await
    }
    /// Detaches the UI while keeping RPC and Neovim active.
    pub async fn detach_ui(&self) -> Result<(), crate::ClientError> {
        self.connection.detach_ui().await
    }
    /// Requests RPC shutdown without killing the child.
    pub fn stop(&mut self) {
        self.connection.stop();
    }
    /// Waits for RPC cleanup without killing the child.
    ///
    /// Cancellation and repeated waits behave as in [`Connection::wait()`].
    pub async fn wait_connection(&mut self) -> Result<(), ConnectionError> {
        self.connection.wait().await
    }

    /// Starts `nvim --embed` and attaches a UI with default limits and no RPC methods.
    ///
    /// Set the executable, arguments, environment, and working directory on
    /// `command`. RPC uses piped stdin/stdout and stderr is inherited. Adds
    /// `--embed` if absent and rejects `--headless`. Use a Tokio runtime with
    /// I/O enabled. Dimensions count cells and must be positive; enable
    /// `ext_linegrid` or an extension that implies it.
    pub async fn spawn(
        command: &mut Command,
        width: u32,
        height: u32,
        options: UiOptions,
    ) -> Result<Self, Error> {
        Self::spawn_with_options(
            command,
            width,
            height,
            options,
            ConnectionOptions::default(),
        )
        .await
    }

    /// Starts an editor with explicit UI and RPC limits.
    pub async fn spawn_with_options(
        command: &mut Command,
        width: u32,
        height: u32,
        options: UiOptions,
        connection_options: ConnectionOptions,
    ) -> Result<Self, Error> {
        Self::spawn_with_handler(
            command,
            width,
            height,
            options,
            NoHandler,
            connection_options,
        )
        .await
    }

    /// Starts an editor with application RPC methods on the UI connection.
    ///
    /// Uses the same process and attachment configuration as [`Self::spawn()`].
    pub async fn spawn_with_handler<H: Handler<ChildStdin>>(
        command: &mut Command,
        width: u32,
        height: u32,
        options: UiOptions,
        handler: H,
        connection_options: ConnectionOptions,
    ) -> Result<Self, Error> {
        crate::input::validate_attachment(width, height, &options)?;
        connection_options.rpc.validate()?;
        tokio::runtime::Handle::try_current().map_err(std::io::Error::other)?;
        if command.as_std().get_args().any(|arg| arg == "--headless") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "UI embedding requires startup without --headless",
            )
            .into());
        }
        if !command.as_std().get_args().any(|arg| arg == "--embed") {
            command.arg("--embed");
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let reader = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("missing Neovim stdout"))?;
        let writer = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("missing Neovim stdin"))?;
        let connection = connect(reader, writer, handler, connection_options)?;
        let mut editor = Self { connection, child };
        if let Err(e) = editor.client().attach(width, height, options).await {
            let _ = editor.shutdown().await;
            return Err(e.into());
        }
        Ok(editor)
    }

    /// Kills and reaps the child, then waits for shared RPC cleanup.
    ///
    /// Does not save modified buffers. Requests stop and process termination
    /// before awaiting. Cancelling leaves cleanup available for retry. The child
    /// is reaped before a connection error is returned.
    pub async fn shutdown(&mut self) -> Result<ExitStatus, Error> {
        self.stop();
        let status = self.child.try_wait()?;
        if status.is_none() {
            self.child.start_kill()?;
        }
        let status = match status {
            Some(status) => status,
            None => self.child.wait().await?,
        };
        self.wait_connection().await?;
        Ok(status)
    }

    /// Waits for the child to exit without requesting termination.
    ///
    /// Does not wait for RPC cleanup. Request exit before waiting if Neovim
    /// would otherwise need user interaction. Cancelling permits another wait.
    pub async fn wait(&mut self) -> Result<ExitStatus, Error> {
        Ok(self.child.wait().await?)
    }
    /// Returns the child process ID while available.
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}
