use crate::rpc::{MessageId, Response};
use crate::{Bytes, ConnectionError, Decode, Error, IntoResponse};

use nvim_rpc::api_types::Params;
use nvim_rpc::mp::encode::TokioWrite;
use nvim_rpc::mp::encode::primitive::{Primitive, State as EncodeState};
use nvim_rpc::sys::types::ApiFn;
use nvim_rpc::{ResponseError, SendError};

use tokio::io::{AsyncWrite, BufWriter};
use tokio::sync::{Mutex, MutexGuard, oneshot, watch};

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, MutexGuard as StdMutexGuard};

type Writer<W> = nvim_rpc::Neovim<TokioWrite<BufWriter<W>>>;
type Waiter = oneshot::Sender<Result<Response, ConnectionError>>;

struct State {
    error: Option<ConnectionError>,
    pending: HashMap<MessageId, Waiter>,
}

struct Shared<W> {
    writer: Mutex<Option<Writer<W>>>,
    state: StdMutex<State>,
    closed: watch::Sender<bool>,
}

impl<W> Shared<W> {
    fn state(&self) -> StdMutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn error(&self) -> Option<ConnectionError> {
        self.state().error.clone()
    }

    fn close(&self, error: ConnectionError) {
        let pending = {
            let mut state = self.state();
            if state.error.is_some() {
                return;
            }
            state.error = Some(error.clone());
            std::mem::take(&mut state.pending)
        };
        self.closed.send_replace(true);
        for waiter in pending.into_values() {
            let _ = waiter.send(Err(error.clone()));
        }
        self.release_writer();
    }

    fn release_writer(&self) {
        if let Ok(mut writer) = self.writer.try_lock() {
            writer.take();
        }
    }
}

/// A cloneable client sharing one serialized writer and a response registry.
///
/// Construct it with [`crate::spawn()`]. Cloning does not require `W: Clone`.
/// Requests release the writer before waiting for their responses, allowing
/// other requests and handler replies to use the same connection.
/// A reusable Tokio buffer combines encoded chunks before writing to `W`;
/// every request, notification, and response is flushed before releasing it.
pub struct Neovim<W> {
    inner: Arc<Shared<W>>,
}

impl<W> Clone for Neovim<W> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<W> Neovim<W> {
    pub(crate) fn new(writer: W) -> Self
    where
        W: AsyncWrite,
    {
        let (closed, _) = watch::channel(false);
        Self {
            inner: Arc::new(Shared {
                writer: Mutex::new(Some(Writer::new(TokioWrite::new(BufWriter::new(writer))))),
                state: StdMutex::new(State {
                    error: None,
                    pending: HashMap::new(),
                }),
                closed,
            }),
        }
    }

    pub(crate) fn close(&self, error: ConnectionError) {
        self.inner.close(error);
    }

    pub(crate) fn connection_error(&self) -> ConnectionError {
        self.inner.error().unwrap_or(ConnectionError::Closed)
    }

    pub(crate) async fn closed(&self) {
        let mut closed = self.inner.closed.subscribe();
        let _ = closed.wait_for(|value| *value).await;
    }

    pub(crate) fn route(&self, response: Response) -> Result<(), ConnectionError> {
        let waiter = {
            let mut state = self.inner.state();
            if let Some(error) = &state.error {
                return Err(error.clone());
            }
            state
                .pending
                .remove(&response.msgid)
                .ok_or(ConnectionError::UnexpectedResponse(response.msgid))?
        };
        let _ = waiter.send(Ok(response));
        Ok(())
    }

    pub(crate) async fn writer(&self) -> Result<WriteGuard<'_, W>, ConnectionError> {
        if let Some(error) = self.inner.error() {
            return Err(error);
        }
        let mut writer = tokio::select! {
            writer = self.inner.writer.lock() => writer,
            () = self.closed() => return Err(self.connection_error()),
        };
        if let Some(error) = self.inner.error() {
            writer.take();
            return Err(error);
        }
        Ok(WriteGuard {
            shared: &self.inner,
            writer: Some(writer),
            active: false,
        })
    }

    fn write_error(&self, error: SendError<std::io::Error>) -> Error {
        match error {
            SendError::IdsExhausted => Error::IdsExhausted,
            SendError::Write(error) => {
                let error = ConnectionError::from(error);
                self.close(error.clone());
                error.into()
            }
            SendError::Failed => {
                self.close(ConnectionError::Interrupted);
                self.connection_error().into()
            }
        }
    }
}

impl<W: AsyncWrite + Unpin + Send + 'static> Neovim<W> {
    /// Writes and flushes an API request, then waits for and decodes its response.
    ///
    /// Results use the owned [`Bytes`] buffer specialization. For example,
    /// request `String` for UTF-8 text or [`crate::Bin`] to keep the original
    /// bytes. Dropping a future waiting for a response does not cancel the
    /// remote operation; its eventual response is still consumed. Cancellation
    /// during a write closes the connection and wakes all waiting requests.
    ///
    /// The first type argument selects the API function; the second selects
    /// the Rust result representation. For example,
    /// `nvim.request::<api::NvimGetCurrentLine, Bin>(()).await` keeps raw bytes,
    /// while `nvim.request::<api::NvimGetCurrentLine, String>(()).await` checks
    /// UTF-8 and returns an owned string. Use `_` when the result type is inferred.
    pub async fn request<F, T>(
        &self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>
        + Send,
    ) -> Result<T, Error>
    where
        F: ApiFn,
        T: Decode<F::Return, Bytes>,
    {
        let mut writer = self.writer().await?;
        let id = writer.sender().next_request_id().ok_or_else(|| {
            if writer.sender().is_failed() {
                self.close(ConnectionError::Interrupted);
                self.connection_error().into()
            } else {
                Error::IdsExhausted
            }
        })?;
        let (waiter, response) = oneshot::channel();
        {
            let mut state = self.inner.state();
            if let Some(error) = &state.error {
                return Err(error.clone().into());
            }
            let previous = state.pending.insert(id, waiter);
            debug_assert!(previous.is_none());
        }
        writer.active = true;
        let pending = tokio::select! {
            result = async {
                let pending = writer.sender().request_async::<F>(params).await?;
                writer.sender().flush_async().await.map_err(SendError::Write)?;
                Ok::<_, SendError<std::io::Error>>(pending)
            } => result.map_err(|error| self.write_error(error))?,
            () = self.closed() => return Err(self.connection_error().into()),
        };
        debug_assert_eq!(id, pending.id());
        writer.active = false;
        drop(writer);

        let response = response.await.map_err(|_| self.connection_error())??;
        pending.decode(response).map_err(|error| match error {
            ResponseError::Remote(value) => Error::Remote(value),
            ResponseError::Decode(error) => Error::Decode(error),
            ResponseError::UnexpectedId { response, .. } => {
                self.close(ConnectionError::UnexpectedResponse(response.msgid));
                self.connection_error().into()
            }
        })
    }

    /// Writes and flushes a notification without expecting a response.
    ///
    /// For example, `nvim.notify::<api::NvimSetCurrentLine>(("hello",)).await`.
    pub async fn notify<F: ApiFn>(
        &self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>
        + Send,
    ) -> Result<(), Error> {
        let mut writer = self.writer().await?;
        writer.active = true;
        tokio::select! {
            result = async {
                writer.sender().notify_async::<F>(params).await?;
                writer.sender().flush_async().await.map_err(SendError::Write)
            } => result.map_err(|error| self.write_error(error))?,
            () = self.closed() => return Err(self.connection_error().into()),
        }
        writer.active = false;
        Ok(())
    }

    pub(crate) async fn respond<R: IntoResponse>(
        &self,
        writer: &mut WriteGuard<'_, W>,
        id: MessageId,
        response: R,
    ) -> Result<(), ConnectionError> {
        writer.active = true;
        tokio::select! {
            result = async {
                writer.sender().send_async(crate::mp::rpc::encode::Response {
                    msgid: id,
                    result: response.into_response(),
                }.to_primitive()).await?;
                writer.sender().flush_async().await.map_err(SendError::Write)
            } => match result {
                Ok(()) => {},
                Err(error) => {
                    self.write_error(error);
                    return Err(self.connection_error());
                }
            },
            () = self.closed() => return Err(self.connection_error()),
        }
        writer.active = false;
        Ok(())
    }
}

pub(crate) struct WriteGuard<'a, W> {
    shared: &'a Shared<W>,
    writer: Option<MutexGuard<'a, Option<Writer<W>>>>,
    active: bool,
}

impl<W> WriteGuard<'_, W> {
    fn sender(&mut self) -> &mut Writer<W> {
        self.writer
            .as_mut()
            .expect("write guard holds the writer lock")
            .as_mut()
            .expect("open connection has a writer")
    }
}

impl<W> Drop for WriteGuard<'_, W> {
    fn drop(&mut self) {
        if self.active {
            self.shared.close(ConnectionError::Interrupted);
        }
        // Let a concurrent close acquire the writer even if it starts after this drop.
        drop(self.writer.take());
        if self.shared.error().is_some() {
            self.shared.release_writer();
        }
    }
}
