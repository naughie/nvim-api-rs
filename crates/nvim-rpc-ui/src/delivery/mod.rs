//! UI frame delivery through an existing plugin RPC dispatcher.
//!
//! Wrap the application's [`Handler`] in [`UiHandler`] and pass it to
//! [`nvim_rpc_plugin::spawn()`]. UI notifications update the retained model;
//! other notifications and all requests reach the application handler. Use the
//! returned [`Neovim`] for UI attachment, input, and ordinary API calls on the
//! same connection. Attach with [`crate::UiOptions::ext_linegrid`] enabled.
//!
//! Receive frames through [`Frames::recv()`] or [`Frames::present_next()`] on
//! the toolkit's owning thread. The backend needs no `Send` or `Sync` bounds.
//! Frame delivery does not wait for drawing. Later frames are coalesced with
//! any unconsumed frame, preserving ordered events and requesting a full
//! repaint. There is at most one pending frame with at most
//! [`Limits::max_pending_events`] events. The model separately limits
//! unflushed events. Application-retained frames are outside these limits.
//!
//! A UI protocol error, a pending-event limit, or invalid application
//! notification arguments closes the shared connection. [`Frames::recv()`]
//! returns the pending valid frame, then the concrete adapter error, then
//! `None`. Await the task returned by `spawn()` to observe transport errors or
//! handler failures; those errors do not appear in this frame stream.

mod error;
pub use error::Error;
mod frames;
pub use frames::Frames;
use frames::Sender;

use crate::{Limits, Model, RedrawError};

use nvim_rpc_plugin::{Array, FromParams, Handler, Neovim};

use tokio::io::AsyncWrite;

use std::sync::{Mutex, MutexGuard};

/// Adds UI notification handling to an application's RPC handler.
///
/// Requests and non-UI notifications retain [`Handler`]'s execution order.
/// `redraw` and `nvim_error_event` notifications are consumed by the UI model
/// before later callbacks are dispatched. Construction starts no tasks.
pub struct UiHandler<H> {
    handler: H,
    model: Mutex<Model>,
    frames: Sender,
}

impl<H> UiHandler<H> {
    /// Wraps a handler and returns its bounded frame receiver.
    ///
    /// Dropping the receiver ends the connection when the next frame is
    /// published. Dropping this handler finishes the stream after its pending
    /// frame. `limits` controls both the model and coalesced event count.
    pub fn new(handler: H, limits: Limits) -> (Self, Frames) {
        let (frames, receiver) = Frames::channel(limits.max_pending_events);
        (
            Self {
                handler,
                model: Mutex::new(Model::new(limits)),
                frames,
            },
            receiver,
        )
    }

    fn model(&self) -> MutexGuard<'_, Model> {
        self.model.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl<W, H> Handler<W> for UiHandler<H>
where
    W: AsyncWrite + Unpin + Send + 'static,
    H: Handler<W>,
{
    type Request = H::Request;
    type Response = H::Response;
    type Notification = Array;

    async fn on_request(
        &self,
        method: &[u8],
        params: Self::Request,
        nvim: Neovim<W>,
    ) -> Self::Response {
        self.handler.on_request(method, params, nvim).await
    }

    async fn on_notification(&self, method: &[u8], params: Array, nvim: Neovim<W>) {
        let args = params.with_borrowed_buffer();
        let result = self
            .model()
            .apply_notification_with(method, args, |frame| self.frames.send(frame));
        let error = match result {
            Ok(true) => return,
            Ok(false) => match H::Notification::from_params(params) {
                Ok(params) => {
                    self.handler.on_notification(method, params, nvim).await;
                    return;
                }
                Err(e) => Error::Notification(e),
            },
            Err(RedrawError::Protocol(e)) => Error::Protocol(e),
            Err(RedrawError::Backend(e)) => e,
        };
        self.frames.fail(error);
        nvim.close();
    }
}
