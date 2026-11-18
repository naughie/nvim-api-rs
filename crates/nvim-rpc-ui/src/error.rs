use nvim_rpc::api_types::DecodeError;

use std::fmt;

/// An error while decoding or applying a redraw notification.
#[derive(Debug)]
pub enum ProtocolError {
    /// An argument cannot be decoded into its UI representation.
    Decode(DecodeError),
    /// An event has invalid arguments or refers to missing UI state.
    Invalid { event: String, reason: &'static str },
    /// A configured resource limit was exceeded.
    Limit(&'static str),
    /// The model cannot accept more updates after an earlier error.
    Failed,
    /// A legacy grid event was received; attach with `ext_linegrid` enabled.
    LegacyGrid,
}

impl From<DecodeError> for ProtocolError {
    fn from(e: DecodeError) -> Self {
        Self::Decode(e)
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(e) => e.fmt(f),
            Self::Invalid { event, reason } => write!(f, "invalid {event} event: {reason}"),
            Self::Limit(name) => write!(f, "UI resource limit exceeded: {name}"),
            Self::Failed => f.write_str("UI model is unusable after an earlier error"),
            Self::LegacyGrid => {
                f.write_str("legacy UI grid protocol is unsupported; enable ext_linegrid")
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

/// A redraw could not be decoded or presented by the backend.
#[derive(Debug)]
pub enum RedrawError<E> {
    /// The notification could not be applied to the UI state.
    Protocol(ProtocolError),
    /// The backend rejected a frame.
    Backend(E),
}

impl<E> From<ProtocolError> for RedrawError<E> {
    fn from(e: ProtocolError) -> Self {
        Self::Protocol(e)
    }
}

impl<E> From<DecodeError> for RedrawError<E> {
    fn from(e: DecodeError) -> Self {
        Self::Protocol(e.into())
    }
}

impl<E: fmt::Display> fmt::Display for RedrawError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Backend(e) => write!(f, "could not present UI frame: {e}"),
        }
    }
}

impl<E: std::error::Error> std::error::Error for RedrawError<E> {}

/// A UI connection, operation, or owned process failed.
#[derive(Debug)]
pub enum Error {
    /// The shared RPC connection failed.
    Connection(nvim_rpc_plugin::ConnectionError),
    /// UI notification processing or frame delivery failed.
    Ui(crate::FrameError),
    /// A UI operation failed locally or in Neovim.
    Client(crate::ClientError),
    /// Process startup or cleanup failed.
    Io(std::io::Error),
}

impl From<nvim_rpc_plugin::ConnectionError> for Error {
    fn from(e: nvim_rpc_plugin::ConnectionError) -> Self {
        Self::Connection(e)
    }
}
impl From<crate::FrameError> for Error {
    fn from(e: crate::FrameError) -> Self {
        Self::Ui(e)
    }
}
impl From<crate::ClientError> for Error {
    fn from(e: crate::ClientError) -> Self {
        Self::Client(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => e.fmt(f),
            Self::Ui(e) => e.fmt(f),
            Self::Client(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}

/// A UI update could not be received or presented.
#[derive(Debug)]
pub enum PresentationError<E> {
    /// The UI or its shared connection failed.
    Session(Error),
    /// The backend rejected a frame.
    Backend(E),
}
impl<E: fmt::Display> fmt::Display for PresentationError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Session(e) => e.fmt(f),
            Self::Backend(e) => write!(f, "could not present UI frame: {e}"),
        }
    }
}
impl<E: std::error::Error> std::error::Error for PresentationError<E> {}
