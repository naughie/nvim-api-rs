//! Tokio plugin handlers and a Neovim client that waits for RPC responses.
//!
//! [`spawn()`] starts the receive loop and handler tasks on the current Tokio
//! runtime. Implement [`Handler`] to receive requests and notifications;
//! [`FromParams`] decodes their arguments and [`IntoResponse`] encodes replies.
//! [`Neovim::request()`] writes and flushes a typed API call, then waits for its
//! matching response. Handler parameters retain shared message storage, so they
//! can remain in use while more messages are received.
//!
//! The `Handler`/`Neovim` interface is inspired by
//! [nvim-rs](https://github.com/KillTheMule/nvim-rs).

pub use bytes::Bytes;
pub use nvim_api::api_types::{Decode, DecodeError};
pub use nvim_api::{
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
    pub use nvim_api::rpc::MessageId;

    /// A request, response, notification, or invalid envelope.
    pub type Message = nvim_api::mp::rpc::decode::Message<bytes::Bytes>;
    /// An incoming request with its method and argument array.
    pub type Request = nvim_api::mp::rpc::decode::Request<bytes::Bytes>;
    /// An incoming response with its request ID and success or error value.
    pub type Response = nvim_api::mp::rpc::decode::Response<bytes::Bytes>;
    /// An incoming notification with its method and argument array.
    pub type Notification = nvim_api::mp::rpc::decode::Notification<bytes::Bytes>;
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
/// use nvim_api_plugin::{Array, Handler, Neovim};
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
    /// Each call runs in a separate task. `nvim` can make nested calls while
    /// the connection continues receiving messages.
    fn on_request(
        &self,
        method: &[u8],
        params: Self::Request,
        nvim: Neovim<W>,
    ) -> impl Future<Output = Self::Response> + Send;

    /// Handles a notification without sending a reply.
    ///
    /// Each call runs in a separate task and may overlap with other hooks.
    fn on_notification(
        &self,
        method: &[u8],
        params: Self::Notification,
        nvim: Neovim<W>,
    ) -> impl Future<Output = ()> + Send;
}
