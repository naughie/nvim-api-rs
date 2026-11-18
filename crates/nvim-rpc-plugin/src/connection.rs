use crate::{ConnectionError, Handler, Neovim};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task::JoinHandle;

use std::sync::Arc;

/// Bounds messages and work retained by a managed connection.
#[derive(Debug, Clone, Copy)]
pub struct ConnectionOptions {
    /// Maximum encoded size of one incoming RPC message, including its envelope.
    pub max_rpc_bytes: usize,
    /// Maximum outgoing requests whose responses have not arrived.
    /// Cancelled response waits still count until their late response arrives.
    pub max_pending_requests: usize,
    /// Maximum incoming requests whose replies have not started writing.
    pub max_pending_replies: usize,
    /// Maximum callbacks queued behind an active notification.
    pub max_pending_callbacks: usize,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            max_rpc_bytes: 64 * 1024 * 1024,
            max_pending_requests: 4096,
            max_pending_replies: 4096,
            max_pending_callbacks: 4096,
        }
    }
}

impl ConnectionOptions {
    pub(crate) const UNLIMITED: Self = Self {
        max_rpc_bytes: usize::MAX,
        max_pending_requests: usize::MAX,
        max_pending_replies: usize::MAX,
        max_pending_callbacks: usize::MAX,
    };

    /// Checks the limits before acquiring a transport or starting tasks.
    ///
    /// Rejects a zero message-byte limit. Zero pending-work limits are valid
    /// and disable admission of the corresponding work.
    pub fn validate(self) -> Result<(), ConnectionError> {
        if self.max_rpc_bytes == 0 {
            return Err(ConnectionError::InvalidOptions(
                "max_rpc_bytes must be positive",
            ));
        }
        Ok(())
    }
}

/// Owns a receive task, its handlers, and the shared RPC client.
///
/// Dropping closes all client clones and aborts the receive task and handlers.
/// Use [`Self::wait()`] to observe completion and finish releasing the writer.
/// Clean EOF closes the client immediately, then drains callbacks already
/// received in dispatch order. Explicit closure discards pending callbacks.
pub struct Connection<W> {
    pub client: Neovim<W>,
    task: ReceiveTask,
    stopped: bool,
}

enum ReceiveTask {
    Running(JoinHandle<Result<(), ConnectionError>>),
    Finished(Result<(), ConnectionError>),
}

impl<W> Connection<W> {
    /// Closes all client clones and stops the receive task and its handlers.
    ///
    /// This method does not wait. Await [`Self::wait()`] for task and writer cleanup.
    pub fn close(&mut self) {
        self.stopped = true;
        self.client.close();
    }

    /// Stops the connection through [`Self::close()`].
    pub fn stop(&mut self) {
        self.close();
    }

    /// Reports whether the receive task has finished; writer cleanup may remain.
    pub fn is_finished(&self) -> bool {
        match &self.task {
            ReceiveTask::Running(task) => task.is_finished(),
            ReceiveTask::Finished(_) => true,
        }
    }

    /// Waits for the receive task, handlers, and writer cleanup.
    ///
    /// Cancellation preserves all progress, and repeated calls return the same
    /// result. An outstanding send future may hold the writer lock: poll or drop
    /// that future to let cleanup finish. Closing wakes all pending requests
    /// immediately, including before this cleanup completes.
    pub async fn wait(&mut self) -> Result<(), ConnectionError> {
        let result = match &mut self.task {
            ReceiveTask::Running(task) => {
                let result = match task.await {
                    Ok(result) => result,
                    Err(e) if e.is_cancelled() => Err(self.client.connection_error()),
                    Err(e) => Err(ConnectionError::Task(Arc::new(e))),
                };
                match result {
                    Err(ConnectionError::Closed) if self.stopped => Ok(()),
                    result => result,
                }
            }
            ReceiveTask::Finished(result) => result.clone(),
        };
        self.task = ReceiveTask::Finished(result.clone());
        self.client.finish_close().await;
        result
    }

    /// Closes immediately, then awaits [`Self::wait()`].
    pub async fn close_and_wait(&mut self) -> Result<(), ConnectionError> {
        self.close();
        self.wait().await
    }
}

impl<W> Drop for Connection<W> {
    fn drop(&mut self) {
        self.close();
        if let ReceiveTask::Running(task) = &self.task {
            task.abort();
        }
    }
}

/// Starts a managed RPC connection with explicit resource bounds.
///
/// The caller supplies a running Tokio runtime and owns the transport's process
/// or server. Notifications remain ordered while response matching progresses.
/// Default options bound each RPC frame to 64 MiB and each pending-work count to
/// 4096. A zero pending-work bound disables admission of that work; frame capacity
/// must be positive. [`crate::spawn()`] retains its unbounded legacy defaults.
pub fn connect<R, W, H>(
    reader: R,
    writer: W,
    handler: H,
    options: ConnectionOptions,
) -> Result<Connection<W>, ConnectionError>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    H: Handler<W>,
{
    options.validate()?;
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(ConnectionError::InvalidOptions(
            "a Tokio runtime is required",
        ));
    }
    let (client, task) = crate::runtime::spawn_with_options(reader, writer, handler, options, true);
    Ok(Connection {
        client,
        task: ReceiveTask::Running(task),
        stopped: false,
    })
}
