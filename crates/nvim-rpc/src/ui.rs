//! Iterates the event groups and repeated calls in a `redraw` notification.
//!
//! [`Redraw`] yields event groups, and [`Event::decode_next()`] decodes repeated
//! calls using a generated descriptor. This example reads grid resize and
//! flush events from a supplied notification:
//!
//! ```
//! use nvim_rpc::rpc::Message;
//! use nvim_rpc::{ui::Redraw, ui_events};
//!
//! let bytes = b"\x93\x02\xa6redraw\x92\x92\xabgrid_resize\x93\x01\x50\x18\x92\xa5flush\x90";
//! let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap()
//! else { panic!("expected notification") };
//! let mut resized = false;
//! let mut flushed = false;
//! for event in Redraw::new(notification).unwrap() {
//!     let mut event = event.unwrap();
//!     if event.is::<ui_events::GridResize>() {
//!         while let Some((grid, width, height)) = event
//!             .decode_next::<ui_events::GridResize, (i64, i64, i64)>()
//!             .unwrap()
//!         {
//!             assert_eq!((grid, width, height), (1, 80, 24));
//!             resized = true;
//!         }
//!     } else if event.is::<ui_events::Flush>() {
//!         assert_eq!(event.decode_next::<ui_events::Flush, ()>().unwrap(), Some(()));
//!         flushed = true;
//!     }
//! }
//! assert!(resized && flushed);
//! ```
//!
//! Unknown groups remain available through [`Event::name()`] and its raw argument
//! iterator. The application maintains grid state and renders updates. See the
//! [Neovim UI protocol](https://neovim.io/doc/user/api-ui-events/) for event
//! meanings and ordering.

use crate::api_types::{Decode, DecodeError};
use crate::rpc::Notification;
use crate::sys::types::UiEvent;

use mp::decode::{Array, Bin, Buf, Object};

/// The event groups of one redraw notification, decoded as they are visited.
pub struct Redraw<B>(Array<B>);

impl<B: Buf<Slice = B>> Redraw<B> {
    /// Checks that the notification is named `redraw` and takes its parameter array.
    pub fn new(notification: Notification<B>) -> Result<Self, DecodeError> {
        if !name_eq(&notification.method, "redraw") {
            return Err(DecodeError::UnexpectedType("redraw notification"));
        }
        Ok(Self::from_params(notification.params))
    }

    /// Iterates an already-dispatched redraw notification's parameter array.
    ///
    /// Use [`Self::new()`] when the notification name has not been checked.
    pub fn from_params(params: Array<B>) -> Self {
        Self(params)
    }
}

impl<B: Buf<Slice = B>> Iterator for Redraw<B> {
    type Item = Result<Event<B>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let group = self.0.next()?;
        Some(match group {
            Object::Array(mut calls) => match calls.next() {
                Some(Object::Str(name)) => Ok(Event { name, calls }),
                _ => Err(DecodeError::UnexpectedType("UI event name")),
            },
            _ => Err(DecodeError::UnexpectedType("UI event array")),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<B: Buf<Slice = B>> ExactSizeIterator for Redraw<B> {}

/// An event name and the remaining repeated calls for that event.
///
/// Iterate raw argument arrays, or use [`Self::decode_next()`] with a generated
/// descriptor such as [`crate::ui_events::GridResize`].
pub struct Event<B> {
    name: Bin<B>,
    calls: Array<B>,
}

impl<B> Event<B> {
    /// The original event name, including names unknown to the generated API.
    pub fn name(&self) -> &Bin<B> {
        &self.name
    }
}

impl<B: Buf> Event<B> {
    /// Compares the event name with descriptor `E` without decoding its arguments.
    pub fn is<E: UiEvent>(&self) -> bool {
        name_eq(&self.name, E::NAME)
    }
}

impl<B: Buf<Slice = B>> Event<B> {
    /// Decodes the next call using `E`'s argument markers and the Rust tuple `T`.
    ///
    /// Returns `Ok(None)` when all calls have been visited. A different event
    /// name returns an error without consuming a call. Tuple decoding accepts
    /// appended fields so newer Neovim versions can extend event arguments.
    pub fn decode_next<E: UiEvent, T: Decode<E::Params, B>>(
        &mut self,
    ) -> Result<Option<T>, DecodeError> {
        if !self.is::<E>() {
            return Err(DecodeError::UnexpectedType(E::NAME));
        }
        self.calls.next().map(T::decode).transpose()
    }
}

impl<B: Buf<Slice = B>> Iterator for Event<B> {
    type Item = Result<Array<B>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.calls.next().map(|args| match args {
            Object::Array(args) => Ok(args),
            _ => Err(DecodeError::UnexpectedType("UI event argument array")),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.calls.size_hint()
    }
}

impl<B: Buf<Slice = B>> ExactSizeIterator for Event<B> {}

fn name_eq<B: Buf>(name: &Bin<B>, expected: &str) -> bool {
    let buf = name.get_ref();
    let mut buf = buf.slice(..buf.len());
    expected.bytes().all(|c| buf.first_byte() == Some(c)) && buf.len() == 0
}
