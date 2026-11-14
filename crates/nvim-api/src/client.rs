use crate::api_types::{Decode, DecodeError, Params};
use crate::rpc::{MessageId, Response};
use crate::sys::types::ApiFn;

use mp::decode::{Buf, Object};
use mp::encode::primitive::State as EncodeState;
use mp::encode::{Primitive, WriterAsync, WriterAsyncExt, WriterSync, WriterSyncExt};

use std::fmt;
use std::marker::PhantomData;

/// Convert a writer's completion value into the result used by [`Neovim`].
///
/// `Result<(), E>` preserves `E`; `()` is infallible. `ControlFlow<E>` uses
/// `Break(E)` as failure, and `Option<()>` uses `None` as failure with error `()`.
pub trait WriteResult: mp::encode::write::WriteResult {
    /// The error reported when writing cannot complete.
    type Error;

    /// Convert successful completion to `Ok(())`, preserving a failure's cause.
    fn into_result(self) -> Result<(), Self::Error>;
}

impl<E> WriteResult for Result<(), E> {
    type Error = E;

    fn into_result(self) -> Self {
        self
    }
}

impl WriteResult for () {
    type Error = std::convert::Infallible;

    fn into_result(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<E> WriteResult for std::ops::ControlFlow<E> {
    type Error = E;

    fn into_result(self) -> Result<(), E> {
        match self {
            Self::Continue(()) => Ok(()),
            Self::Break(e) => Err(e),
        }
    }
}

impl WriteResult for Option<()> {
    type Error = ();

    fn into_result(self) -> Result<(), ()> {
        self.ok_or(())
    }
}

/// Arguments `P` associated with API function descriptor `F`.
///
/// Stores arguments for explicit RPC envelope encoding without performing I/O.
/// `F` supplies the RPC name and parameter/return markers. For ordinary sends,
/// pass the descriptor and arguments directly to [`Neovim::request`] or
/// [`Neovim::notify`]. Handle methods can construct a `Call` for manual encoding.
pub struct Call<F, P> {
    params: P,
    function: PhantomData<fn() -> F>,
}

impl<F: ApiFn, P: Params<F::Params>> Call<F, P> {
    /// Build a call whose arguments implement the descriptor's parameter signature.
    pub fn new(params: P) -> Self {
        Self {
            params,
            function: PhantomData,
        }
    }

    /// Recover the arguments without encoding them.
    pub fn into_params(self) -> P {
        self.params
    }

    /// Encode a request envelope using the supplied message ID.
    ///
    /// The converted parameters, encoding state, and chunks must be `Send`.
    /// Use this when assigning IDs and writing messages outside [`Neovim`].
    pub fn into_request(
        self,
        msgid: MessageId,
    ) -> impl Primitive<State: EncodeState<Item: Send> + Send> + Send
    where
        P::Encoded: Send,
        <P::Encoded as Primitive>::State: Send,
        <<P::Encoded as Primitive>::State as EncodeState>::Item: Send,
    {
        mp::encode::rpc::Request {
            msgid,
            method: F::NAME,
            params: self.params.encode(),
        }
        .to_primitive()
    }

    /// Encode a notification envelope, which has no message ID or response.
    ///
    /// The encoding has the same `Send` requirements as [`Self::into_request`].
    pub fn into_notification(self) -> impl Primitive<State: EncodeState<Item: Send> + Send> + Send
    where
        P::Encoded: Send,
        <P::Encoded as Primitive>::State: Send,
        <<P::Encoded as Primitive>::State as EncodeState>::Item: Send,
    {
        mp::encode::rpc::Notification {
            method: F::NAME,
            params: self.params.encode(),
        }
        .to_primitive()
    }
}

/// A request ID paired with the expected return marker from function `F`.
///
/// Route the matching response from the same connection to [`Self::decode`].
/// The token is copyable and retains its ID after decoding.
///
/// ```
/// use nvim_api::mp::decode::Object;
/// use nvim_api::rpc::{MessageId, Response};
/// use nvim_api::{Buffer, Pending, sys};
///
/// let pending = Pending::<sys::functions::NvimGetCurrentBuf>::new(MessageId(3));
/// let response = Response::<&[u8]> {
///     msgid: MessageId(3),
///     result: Ok(Object::U8(7)),
/// };
/// let buffer: Buffer = pending.decode(response).unwrap();
/// assert_eq!(buffer, Buffer(7));
/// ```
pub struct Pending<F> {
    id: MessageId,
    function: PhantomData<fn() -> F>,
}

impl<F> Copy for Pending<F> {}
impl<F> Clone for Pending<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F> fmt::Debug for Pending<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pending").field(&self.id).finish()
    }
}

impl<F> Pending<F> {
    /// Construct a token for a request whose ID was assigned by the application.
    pub const fn new(id: MessageId) -> Self {
        Self {
            id,
            function: PhantomData,
        }
    }

    /// The request ID expected by this token.
    pub const fn id(self) -> MessageId {
        self.id
    }

    /// Check the response ID without consuming or decoding the response.
    pub fn matches<B>(&self, response: &Response<B>) -> bool {
        self.id == response.msgid
    }
}

impl<F: ApiFn> Pending<F> {
    /// Check the response ID, propagate a remote error, and decode its value as `T`.
    ///
    /// A mismatched response is returned intact in
    /// [`ResponseError::UnexpectedId`]. Borrowed results and remote errors retain
    /// the lifetime of the response's backing buffer.
    pub fn decode<B: Buf, T: Decode<F::Return, B>>(
        self,
        response: Response<B>,
    ) -> Result<T, ResponseError<B>> {
        if !self.matches(&response) {
            return Err(ResponseError::UnexpectedId {
                expected: self.id,
                response,
            });
        }
        let value = response.result.map_err(ResponseError::Remote)?;
        T::decode(value).map_err(ResponseError::Decode)
    }
}

/// A response could not be matched or decoded, or Neovim returned an error.
#[derive(Debug)]
pub enum ResponseError<B> {
    /// The response belongs to another request; its payload remains available.
    UnexpectedId {
        expected: MessageId,
        response: Response<B>,
    },
    /// Neovim's error value, preserved in its original representation.
    Remote(Object<B>),
    /// The returned value does not match the requested Rust representation.
    Decode(DecodeError),
}

impl<B> fmt::Display for ResponseError<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedId { expected, response } => write!(
                f,
                "expected response {}, received {}",
                expected.0, response.msgid.0
            ),
            Self::Remote(_) => f.write_str("Neovim returned an RPC error"),
            Self::Decode(e) => e.fmt(f),
        }
    }
}

impl<B: fmt::Debug> std::error::Error for ResponseError<B> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(e) => Some(e),
            _ => None,
        }
    }
}

/// A write failed, the request ID range was exhausted, or the sender is unusable.
#[derive(Debug)]
pub enum SendError<E> {
    /// The writer failed; the connection may contain an incomplete RPC frame.
    Write(E),
    /// All unsigned 32-bit request IDs have been used; IDs never wrap.
    IdsExhausted,
    /// An earlier failed or interrupted write left the sender unusable.
    Failed,
}

impl<E: fmt::Display> fmt::Display for SendError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write(e) => write!(f, "could not write RPC message: {e}"),
            Self::IdsExhausted => f.write_str("RPC request IDs exhausted"),
            Self::Failed => f.write_str("RPC sender is unusable after an interrupted write"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SendError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write(e) => Some(e),
            _ => None,
        }
    }
}

enum State {
    Ready(Option<MessageId>),
    Failed,
}

/// The sending side of a Neovim connection, parameterized by its writer.
///
/// Requests consume increasing IDs; notifications and raw messages do not:
///
/// ```
/// use nvim_api::{api, Neovim};
///
/// let mut nvim = Neovim::new(Vec::new());
/// let first = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
/// nvim.notify::<api::NvimSetCurrentLine>(("hello",)).unwrap();
/// let second = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
/// assert_eq!(first.id().0, 0);
/// assert_eq!(second.id().0, 1);
/// ```
///
/// The sender owns no reader, response registry, or background task. Receive
/// and dispatch messages with [`crate::Receiver`] or the application's parser.
/// Access through `&mut self` keeps writes through this sender sequential.
pub struct Neovim<W> {
    writer: W,
    state: State,
}

impl<W> Neovim<W> {
    /// Start a sender whose first request ID is zero.
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            state: State::Ready(Some(MessageId(0))),
        }
    }

    /// Borrow the writer, for example to inspect a memory buffer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Mutably borrow the writer for backend-specific operations.
    ///
    /// Writing unrelated bytes here can invalidate the RPC stream.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Flush buffered output, returning the writer's result.
    ///
    /// Flush after writing a request and before waiting for its response. This
    /// forwards to the writer; it does not wait for acknowledgement from Neovim.
    pub fn flush(&mut self) -> W::Result
    where
        W: WriterSync,
    {
        WriterSync::flush(&mut self.writer)
    }

    /// Flush buffered output asynchronously, returning the writer's result.
    ///
    /// Like [`Self::flush`], this forwards to the writer without waiting for a
    /// response from Neovim.
    pub async fn flush_async(&mut self) -> W::Result
    where
        W: WriterAsync,
    {
        WriterAsync::flush(&mut self.writer).await
    }

    /// Recover the writer without flushing it.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Whether an earlier interrupted write prevents further sends.
    pub fn is_failed(&self) -> bool {
        matches!(self.state, State::Failed)
    }

    /// The ID that the next request will use, if the sender can issue a request.
    ///
    /// A response dispatcher can register a waiter before starting a write.
    /// The ID is not reserved; keep exclusive access until calling `request` or
    /// `request_async`. Returns `None` after failure or ID exhaustion.
    pub fn next_request_id(&self) -> Option<MessageId> {
        match self.state {
            State::Ready(id) => id,
            State::Failed => None,
        }
    }

    fn begin<E>(&mut self) -> Result<Option<MessageId>, SendError<E>> {
        match std::mem::replace(&mut self.state, State::Failed) {
            State::Ready(next) => Ok(next),
            State::Failed => Err(SendError::Failed),
        }
    }

    fn begin_request<E>(&mut self) -> Result<(MessageId, Option<MessageId>), SendError<E>> {
        match self.state {
            State::Ready(Some(id)) => {
                self.state = State::Failed;
                Ok((id, id.0.checked_add(1).map(MessageId)))
            }
            State::Ready(None) => Err(SendError::IdsExhausted),
            State::Failed => Err(SendError::Failed),
        }
    }
}

impl<W> Neovim<W> {
    /// Write a request and return its typed response token.
    ///
    /// Select a function descriptor from [`crate::api`] and pass its arguments
    /// as a tuple: `()` for no arguments, `(value,)` for one argument.
    /// Arguments retain their Rust representations until encoding.
    ///
    /// Flush a buffered writer before waiting for input, then route the matching
    /// response to [`Pending::decode`]. Encoded parameters, states, and chunks
    /// must be `Send`, including when using a synchronous writer.
    ///
    /// ```
    /// use nvim_api::{Neovim, api};
    ///
    /// let mut nvim = Neovim::new(Vec::new());
    /// let pending = nvim.request::<api::NvimSetCurrentLine>(("hello",)).unwrap();
    /// ```
    ///
    /// The descriptor checks argument types before any I/O:
    ///
    /// ```compile_fail
    /// use nvim_api::{Neovim, api};
    /// let mut nvim = Neovim::new(Vec::new());
    /// nvim.request::<api::NvimSetCurrentLine>((123i64,)).unwrap();
    /// ```
    pub fn request<F: ApiFn>(
        &mut self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>,
    ) -> Result<Pending<F>, SendError<<W::Result as WriteResult>::Error>>
    where
        W: WriterSync,
        W::Result: WriteResult,
    {
        let (id, next) = self.begin_request()?;
        WriterSyncExt::write_primitive(
            &mut self.writer,
            Call::<F, _>::new(params).into_request(id),
        )
        .consume()
        .into_result()
        .map_err(SendError::Write)?;
        self.state = State::Ready(next);
        Ok(Pending::new(id))
    }

    /// Write a notification without consuming a request ID or expecting a reply.
    ///
    /// Like [`Self::request`], select the function with a type argument and pass
    /// its parameters as a tuple, for example
    /// `nvim.notify::<api::NvimSetCurrentLine>(("hello",))`.
    pub fn notify<F: ApiFn>(
        &mut self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>,
    ) -> Result<(), SendError<<W::Result as WriteResult>::Error>>
    where
        W: WriterSync,
        W::Result: WriteResult,
    {
        self.send(Call::<F, _>::new(params).into_notification())
    }

    /// Write a primitive, such as an encoded RPC response, without assigning an ID.
    ///
    /// See [`crate::rpc`] for a plugin reply example. Raw primitive sending does
    /// not add `Send` bounds to the primitive or its state.
    pub fn send<P: Primitive, E>(&mut self, value: P) -> Result<(), SendError<E>>
    where
        W: WriterSync,
        W::Result: WriteResult<Error = E>,
    {
        let next = self.begin()?;
        WriterSyncExt::write_primitive(&mut self.writer, value)
            .consume()
            .into_result()
            .map_err(SendError::Write)?;
        self.state = State::Ready(next);
        Ok(())
    }

    /// Write a request asynchronously, then return its response token.
    ///
    /// Dropping this future before polling has no effect. Once writing starts,
    /// failure or cancellation leaves this sender unusable: later writes could
    /// otherwise follow an incomplete frame. Awaiting a response is a separate
    /// receive/dispatch step after this method completes.
    pub async fn request_async<F: ApiFn>(
        &mut self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>,
    ) -> Result<Pending<F>, SendError<<W::Result as WriteResult>::Error>>
    where
        W: WriterAsync,
        W::Result: WriteResult,
    {
        let (id, next) = self.begin_request()?;
        WriterAsyncExt::write_primitive(
            &mut self.writer,
            Call::<F, _>::new(params).into_request(id),
        )
        .consume_async()
        .await
        .into_result()
        .map_err(SendError::Write)?;
        self.state = State::Ready(next);
        Ok(Pending::new(id))
    }

    /// Write a notification asynchronously, with no request ID or reply.
    ///
    /// Interrupted writes follow the failure policy of [`Self::request_async`].
    pub async fn notify_async<F: ApiFn>(
        &mut self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: EncodeState<Item: Send> + Send>>,
    ) -> Result<(), SendError<<W::Result as WriteResult>::Error>>
    where
        W: WriterAsync,
        W::Result: WriteResult,
    {
        self.send_async(Call::<F, _>::new(params).into_notification())
            .await
    }

    /// Write a primitive asynchronously without changing the next request ID.
    ///
    /// Interrupted writes follow the failure policy of [`Self::request_async`].
    pub async fn send_async<P: Primitive, E>(&mut self, value: P) -> Result<(), SendError<E>>
    where
        W: WriterAsync,
        W::Result: WriteResult<Error = E>,
    {
        let next = self.begin()?;
        WriterAsyncExt::write_primitive(&mut self.writer, value)
            .consume_async()
            .await
            .into_result()
            .map_err(SendError::Write)?;
        self.state = State::Ready(next);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api;

    #[test]
    fn request_ids_do_not_wrap_and_notifications_remain_available() {
        let mut nvim = Neovim::new(Vec::new());
        nvim.state = State::Ready(Some(MessageId(u32::MAX)));
        let pending = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
        assert_eq!(pending.id(), MessageId(u32::MAX));
        assert!(matches!(
            nvim.request::<api::NvimGetCurrentLine>(()),
            Err(SendError::IdsExhausted)
        ));
        nvim.notify::<api::NvimGetCurrentLine>(()).unwrap();
        assert!(!nvim.is_failed());
    }
}
