//! Tokio plugin handlers and a Neovim client that waits for RPC responses.
//!
//! [`connect()`] starts a managed connection with resource bounds and repeatable
//! shutdown waits. [`spawn()`] returns the receive task directly. Both use the
//! current Tokio runtime. Implement [`Handler`] to receive requests and notifications;
//! [`FromParams`] decodes their arguments and [`IntoResponse`] encodes replies.
//! [`Neovim::request()`] writes and flushes a typed API call, then waits for its
//! matching response. Handler parameters retain shared message storage, so they
//! can remain in use while more messages are received.
//!
//! The `Handler`/`Neovim` interface is inspired by
//! [nvim-rs](https://github.com/KillTheMule/nvim-rs).

pub use bytes::Bytes;
pub use nvim_rpc::api_types::{Decode, DecodeError};
pub use nvim_rpc::{
    API_COMPATIBLE, API_LEVEL, API_PRERELEASE, Buffer, Tabpage, UiOptions, VERSION, Window, api,
    api_types, mp, sys, ui, ui_events,
};

/// A decoded string or binary payload backed by immutable shared bytes.
pub type Bin = mp::decode::Bin<Bytes>;
/// A lazily decoded array backed by immutable shared bytes.
pub type Array = mp::decode::Array<Bytes>;
/// A lazily decoded map backed by immutable shared bytes.
pub type Map = mp::decode::Map<Bytes>;
/// A decoded MessagePack value backed by immutable shared bytes.
pub type Object = mp::decode::Object<Bytes>;

/// Incoming RPC messages backed by shared message storage.
pub mod rpc {
    pub use nvim_rpc::rpc::MessageId;

    /// A request, response, notification, or invalid envelope.
    pub type Message = nvim_rpc::mp::rpc::decode::Message<bytes::Bytes>;
    /// An incoming request with its method and argument array.
    pub type Request = nvim_rpc::mp::rpc::decode::Request<bytes::Bytes>;
    /// An incoming response with its request ID and success or error value.
    pub type Response = nvim_rpc::mp::rpc::decode::Response<bytes::Bytes>;
    /// An incoming notification with its method and argument array.
    pub type Notification = nvim_rpc::mp::rpc::decode::Notification<bytes::Bytes>;
}

mod error;
pub use error::{ConnectionError, Error};

mod params;
pub use params::FromParams;

mod response;
pub use response::IntoResponse;

mod client;
pub use client::Neovim;

mod frame;
mod runtime;
pub use runtime::spawn;
mod connection;
pub use connection::{Connection, ConnectionOptions, connect};

use tokio::io::AsyncWrite;

/// Application callbacks for a connection with Tokio writer `W`.
///
/// Implement both hooks with ordinary `async fn`s. Requests receive exactly
/// one reply; notifications receive none. Use [`Array`] to inspect different
/// parameter layouts by method, or a tuple such as `(i32, i32, Bin)` for a fixed
/// layout. Use `Response = ()` and an empty request body to return a successful
/// empty array. For a nil result, return [`mp::encode::primitive::Nil`] as the
/// success value of a `Result` response.
///
/// The writer and handler are shared across spawned tasks, so `W` must be
/// `Send + 'static` and the handler must be `Send + Sync + 'static`.
///
/// ```
/// use nvim_rpc_plugin::{Array, Handler, Neovim};
/// use tokio::io::AsyncWrite;
///
/// struct Ignore;
///
/// impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Ignore {
///     type Request = Array;
///     type Response = ();
///     type Notification = Array;
///
///     async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}
///     async fn on_notification(&self, _: &[u8], _: Array, _: Neovim<W>) {}
/// }
/// ```
///
/// # Execution order
///
/// Requests and notifications are dispatched in receipt order. Each request
/// spawns a separate task without waiting for earlier requests. Request tasks
/// may overlap; their start and completion order depends on Tokio scheduling,
/// and they may run in parallel on a multithreaded runtime.
///
/// Notifications run one at a time. A notification finishes before any later
/// request or notification is dispatched. It does not wait for earlier request
/// tasks to finish, so those tasks may overlap with it. The receive loop keeps
/// reading and matching responses, allowing both hooks to await nested calls
/// through [`Neovim::request()`].
///
/// For a batch `request A, notification B, request C, notification D`, the
/// dispatcher spawns A and waits for B to finish before spawning C and running D.
/// A may still be running during B, C, or D; C may still be running during D.
/// The same rules apply when messages arrive in separate reads. This dispatch
/// order follows [nvim-rs](https://github.com/KillTheMule/nvim-rs/blob/4c85d61cc6fadc64fb67d8eb46e908f5a8723396/src/rpc/handler.rs#L33-L43).
///
/// Request replies follow Neovim's reverse request order, even when handlers
/// finish in a different order. A request queued behind a notification is
/// registered on receipt, so an earlier request's reply may wait for that queued
/// request to finish too. Argument decoding also waits for earlier notifications;
/// a malformed request cannot send its error reply ahead of them.
pub trait Handler<W>: Send + Sync + 'static
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    /// Arguments decoded before calling [`Self::on_request()`].
    ///
    /// A decoding failure sends an error reply without calling the hook.
    type Request: FromParams;
    /// The success or error value sent as the request's reply.
    type Response: IntoResponse;
    /// Arguments decoded before calling [`Self::on_notification()`].
    ///
    /// A decoding failure ends the connection.
    type Notification: FromParams;

    /// Handles a request and returns its reply.
    ///
    /// Dispatch waits for earlier notifications to finish, then spawns a
    /// separate task without waiting for other requests. Request tasks may
    /// overlap each other and later notifications; their start and completion
    /// order depends on scheduling. `nvim` can make nested calls while the
    /// connection continues receiving messages.
    fn on_request(
        &self,
        method: &[u8],
        params: Self::Request,
        nvim: Neovim<W>,
    ) -> impl Future<Output = Self::Response> + Send;

    /// Handles a notification without sending a reply.
    ///
    /// Notifications run one at a time in receipt order. Each call finishes
    /// before any later request or notification is dispatched, but does not
    /// wait for earlier request tasks to finish. Receiving and response
    /// matching continue, so `nvim` can make nested calls from this hook.
    fn on_notification(
        &self,
        method: &[u8],
        params: Self::Notification,
        nvim: Neovim<W>,
    ) -> impl Future<Output = ()> + Send;
}

/// Ignores notifications and rejects requests with a method error.
///
/// Use this handler when a connection only sends calls to Neovim.
pub struct NoHandler;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for NoHandler {
    type Request = Array;
    type Response = Result<std::convert::Infallible, &'static str>;
    type Notification = Array;

    async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) -> Self::Response {
        Err("RPC client does not implement this method")
    }

    async fn on_notification(&self, _: &[u8], _: Array, _: Neovim<W>) {}
}
