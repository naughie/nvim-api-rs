use crate::ProtocolError;

use nvim_rpc_plugin::DecodeError;

use std::fmt;

/// A UI notification or its delivery could not be processed.
#[derive(Debug)]
pub enum Error {
    /// UI notification decoding or resource-limit checking failed.
    Protocol(ProtocolError),
    /// An application notification's arguments could not be decoded.
    Notification(DecodeError),
    /// The application's frame receiver was dropped.
    ReceiverClosed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Notification(e) => write!(f, "invalid notification arguments: {e}"),
            Self::ReceiverClosed => f.write_str("UI frame receiver was dropped"),
        }
    }
}

impl std::error::Error for Error {}
