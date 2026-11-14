//! Neovim RPC calls with caller-selected arguments, results, and I/O.
//!
//! Select a function descriptor from [`api`] and pass its arguments to
//! [`Neovim::request()`] or [`Neovim::notify()`]. [`Receiver`] reads requests,
//! responses, and notifications; the application owns response dispatch.
//!
//! Use [nvim-rpc-plugin](https://docs.rs/nvim-rpc-plugin) for Tokio plugin
//! handlers and automatic response matching.
//!
//! # Write borrowed arguments
//!
//! This writes a request into a fixed buffer. Neither the line array nor its
//! strings need to be copied into a `Vec<String>`:
//!
//! ```
//! use nvim_rpc::mp::encode::StdWrite;
//! use nvim_rpc::{api, Buffer, Neovim};
//! use std::io::Cursor;
//!
//! let lines = ["first", "日本語"];
//! let mut storage = [0; 256];
//! let mut output = Cursor::new(&mut storage[..]);
//! let mut nvim = Neovim::new(StdWrite::new(&mut output));
//! let pending = nvim
//!     .request::<api::NvimBufSetLines>((Buffer::CURRENT, 0i64, -1i64, true, &lines[..]))
//!     .unwrap();
//! assert_eq!(pending.id().0, 0);
//! assert!(output.position() > 0);
//! ```
//!
//! The returned [`Pending`] identifies the expected response. Route that
//! response to [`Pending::decode()`] in your receive loop.
//!
//! # Read and decode a response
//!
//! Here the input is an encoded response containing `"hello"`. A real
//! connection supplies the same bytes through a socket or a process pipe:
//!
//! ```
//! use nvim_rpc::read::StdRead;
//! use nvim_rpc::rpc::Message;
//! use nvim_rpc::{api, Neovim, Receiver};
//!
//! let mut nvim = Neovim::new(Vec::new());
//! let pending = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
//! nvim.flush();
//! let input = &b"\x94\x01\x00\xc0\xa5hello"[..];
//! let mut receiver = Receiver::new(StdRead(input), [0; 64]);
//! let Message::Response(response) = receiver.recv().unwrap() else {
//!     panic!("expected response");
//! };
//! let line: &str = pending.decode(response).unwrap();
//! assert_eq!(line, "hello");
//! ```
//!
//! `line` borrows the receiver's storage. Choose `String` instead to retain it
//! after receiving another message. [`api_types::decode::Decode`] also supports
//! raw bytes, lazy arrays, and application-defined result types.
//!
//! # Choose a transport
//!
//! | I/O | Reader | Writer | Feature |
//! | --- | --- | --- | --- |
//! | `std::io` | [`read::StdRead`] | [`mp::encode::StdWrite`] | none |
//! | Tokio | `read::TokioRead` | `mp::encode::TokioWrite` | `tokio` |
//! | futures I/O | `read::FuturesRead` | `mp::encode::FuturesWrite` | `futures` |
//!
//! Use `request_async`, `notify_async`, and `recv_async` with asynchronous
//! adapters. Enable `bytes` for the `bytes` buffer adapters in [`mp`]. Features
//! add adapters; they do not start a runtime or spawn tasks. Use [`Neovim::flush()`]
//! or [`Neovim::flush_async()`] before waiting for a response on buffered I/O.
//!
//! See [`api_types::encode`] for custom arguments, [`rpc`] for plugin
//! replies, and [`ui`] for redraw notifications.

pub use mp;
pub use nvim_rpc_sys as sys;
pub use sys::UiOptions;
pub use sys::{API_COMPATIBLE, API_LEVEL, API_PRERELEASE, VERSION, ui_events};

mod generated;
pub use generated::*;

pub mod api_types;

pub mod handles;
pub use handles::{Buffer, Tabpage, Window};

mod client;
pub use client::{Call, Neovim, Pending, ResponseError, SendError, WriteResult};

pub mod read;
pub use read::{ReadError, Receiver};

pub mod ui;

/// Decodes RPC messages and accesses their request IDs and payloads.
///
/// Send a plugin reply with [`Neovim::send()`] and the incoming request's ID:
///
/// ```
/// use nvim_rpc::mp::encode::rpc::Response;
/// use nvim_rpc::rpc::{Message, MessageId};
/// use nvim_rpc::Neovim;
///
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.send(Response {
///     msgid: MessageId(7),
///     result: Ok::<_, &str>(42i64),
/// }.to_primitive()).unwrap();
/// let bytes = nvim.into_inner();
/// let Message::Response(response) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap() else {
///     panic!("expected response");
/// };
/// assert_eq!(response.msgid, MessageId(7));
/// assert!(response.result.is_ok());
/// ```
///
/// This module parses messages and re-exports their payload types. The
/// application routes requests and responses; use
/// [nvim-rpc-plugin](https://docs.rs/nvim-rpc-plugin) for Tokio handler dispatch
/// and response matching. See [Neovim's RPC documentation](https://neovim.io/doc/user/api/#api-rpc)
/// for request ordering requirements.
pub mod rpc {
    use mp::decode::{Buf, InsufficientBytes, ObjectParser};
    pub use mp::rpc::MessageId;
    pub use mp::rpc::decode::{Message, Notification, Request, Response};

    /// Reads one complete RPC message, advancing `buf` past it.
    ///
    /// Returns `InsufficientBytes` for an incomplete MessagePack object, or
    /// [`Message::InvalidFormat`] for a complete object with an invalid RPC
    /// layout. Use [`crate::Receiver`] to accumulate fragmented input.
    pub fn decode<B: Buf>(
        buf: &mut B,
    ) -> Result<Message<<B::Slice as Buf>::Slice>, InsufficientBytes> {
        ObjectParser::new(buf).next_object().map(Into::into)
    }
}
