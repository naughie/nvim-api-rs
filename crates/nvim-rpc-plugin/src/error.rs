use crate::{DecodeError, Object, rpc::MessageId};

use std::fmt;
use std::io;
use std::sync::Arc;

/// A connection failure shared by all requests waiting on that connection.
#[derive(Debug, Clone)]
pub enum ConnectionError {
    Io(Arc<io::Error>),
    Eof { buffered: usize },
    InvalidFormat,
    UnexpectedResponse(MessageId),
    Decode(DecodeError),
    Task(Arc<tokio::task::JoinError>),
    Interrupted,
    Closed,
    Limit(&'static str),
    InvalidOptions(&'static str),
}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Eof { buffered } => {
                write!(f, "RPC stream ended with {buffered} incomplete bytes")
            }
            Self::InvalidFormat => f.write_str("invalid RPC message"),
            Self::UnexpectedResponse(id) => write!(f, "unexpected RPC response {}", id.0),
            Self::Decode(e) => write!(f, "could not decode notification parameters: {e}"),
            Self::Task(e) => write!(f, "plugin task failed: {e}"),
            Self::Interrupted => f.write_str("RPC write was interrupted"),
            Self::Closed => f.write_str("RPC connection is closed"),
            Self::Limit(s) => write!(f, "RPC connection limit exceeded: {s}"),
            Self::InvalidOptions(s) => write!(f, "invalid RPC connection options: {s}"),
        }
    }
}

// Preserving the existing cause chain requires std::error::Error::source().
// That standard trait fixes its return type to dyn Error: concrete enums,
// generics, and opaque return types cannot implement the required signature.
// Omitting this method would remove the established source() behavior.
impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e.as_ref()),
            Self::Decode(e) => Some(e),
            Self::Task(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// A call failed locally, the connection ended, or Neovim returned an error.
#[derive(Debug)]
pub enum Error {
    Connection(ConnectionError),
    Remote(Object),
    Decode(DecodeError),
    IdsExhausted,
}

impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Self::Connection(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => e.fmt(f),
            Self::Remote(_) => f.write_str("Neovim returned an RPC error"),
            Self::Decode(e) => e.fmt(f),
            Self::IdsExhausted => f.write_str("RPC request IDs exhausted"),
        }
    }
}

// The standard source() signature likewise requires dyn Error here to retain
// this error's existing cause chain; a concrete or opaque return cannot replace it.
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::Decode(e) => Some(e),
            _ => None,
        }
    }
}
