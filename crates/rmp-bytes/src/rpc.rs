//! MessagePack-RPC envelopes, enabled by the `rpc` feature.
//!
//! Encode a request, then inspect its borrowed method name and parameters:
//!
//! ```
//! use rmp_bytes::decode::ObjectParser;
//! use rmp_bytes::encode::WriterSyncExt as _;
//! use rmp_bytes::rpc::{MessageId, decode::Message, encode::Request};
//!
//! let mut bytes = Vec::new();
//! bytes.write_primitive(Request {
//!     msgid: MessageId(7),
//!     method: "sum",
//!     params: (20u8, 22u8),
//! }.to_primitive()).consume();
//! assert_eq!(bytes, b"\x94\x00\x07\xa3sum\x92\x14\x16");
//!
//! let message = ObjectParser::new(bytes.as_slice()).next_object()?.into();
//! let Message::Request(mut request) = message else { panic!("expected a request") };
//! assert_eq!(request.msgid, MessageId(7));
//! assert_eq!(request.method.as_str()?, "sum");
//! assert_eq!(request.params.next().unwrap().to_u8(), Some(20));
//! assert_eq!(request.params.next().unwrap().to_u8(), Some(22));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The envelope types handle serialization and parsing only. Assign IDs,
//! match responses, dispatch incoming calls, and manage transport lifetimes in
//! the application or a higher-level client.

/// A MessagePack-RPC message ID in the unsigned 32-bit range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(pub u32);

pub use crate::decode::rpc as decode;
pub use crate::encode::rpc as encode;
