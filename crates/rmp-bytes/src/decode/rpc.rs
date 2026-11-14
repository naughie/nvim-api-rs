use super::Buf;
use super::{Array, Bin, Object};

pub use crate::rpc::MessageId;

impl MessageId {
    fn from_obj(obj: Object<impl Buf>) -> Option<Self> {
        match obj {
            Object::U8(n) => Some(Self(n as u32)),
            Object::U16(n) => Some(Self(n as u32)),
            Object::U32(n) => Some(Self(n)),
            Object::U64(n @ 0..=0xffff_ffff) => Some(Self(n as u32)),

            Object::I8(n @ 0..=0x7f) => Some(Self(n as u32)),
            Object::I16(n @ 0..=0x7fff) => Some(Self(n as u32)),
            Object::I32(n @ 0..=0x7fff_ffff) => Some(Self(n as u32)),
            Object::I64(n @ 0..=0xffff_ffff) => Some(Self(n as u32)),

            _ => None,
        }
    }
}

enum MessageType {
    Request,
    Response,
    Notification,
    Unknown,
}

impl MessageType {
    fn from_obj(obj: Object<impl Buf>) -> MessageType {
        match obj {
            Object::U8(0) => MessageType::Request,
            Object::U8(1) => MessageType::Response,
            Object::U8(2) => MessageType::Notification,

            Object::U16(0) => MessageType::Request,
            Object::U16(1) => MessageType::Response,
            Object::U16(2) => MessageType::Notification,

            Object::U32(0) => MessageType::Request,
            Object::U32(1) => MessageType::Response,
            Object::U32(2) => MessageType::Notification,

            Object::U64(0) => MessageType::Request,
            Object::U64(1) => MessageType::Response,
            Object::U64(2) => MessageType::Notification,

            Object::I8(0) => MessageType::Request,
            Object::I8(1) => MessageType::Response,
            Object::I8(2) => MessageType::Notification,

            Object::I16(0) => MessageType::Request,
            Object::I16(1) => MessageType::Response,
            Object::I16(2) => MessageType::Notification,

            Object::I32(0) => MessageType::Request,
            Object::I32(1) => MessageType::Response,
            Object::I32(2) => MessageType::Notification,

            Object::I64(0) => MessageType::Request,
            Object::I64(1) => MessageType::Response,
            Object::I64(2) => MessageType::Notification,

            _ => MessageType::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Message<B> {
    Request(Request<B>),
    Response(Response<B>),
    Notification(Notification<B>),
    InvalidFormat,
}

impl<B: Buf> From<Object<B>> for Message<B::Slice> {
    fn from(value: Object<B>) -> Self {
        parse(value)
    }
}

fn parse<B: Buf>(msg: Object<B>) -> Message<B::Slice> {
    let Object::Array(mut msg) = msg else {
        return Message::InvalidFormat;
    };
    let msg_len = msg.remaining_len();
    if msg_len != 3 && msg_len != 4 {
        return Message::InvalidFormat;
    }

    let Some(type_id) = msg.next().map(MessageType::from_obj) else {
        return Message::InvalidFormat;
    };

    match type_id {
        MessageType::Request if msg_len == 4 => {
            if let Some(req) = parse_req(msg) {
                Message::Request(req)
            } else {
                Message::InvalidFormat
            }
        }
        MessageType::Response if msg_len == 4 => {
            if let Some(res) = parse_res(msg) {
                Message::Response(res)
            } else {
                Message::InvalidFormat
            }
        }
        MessageType::Notification if msg_len == 3 => {
            if let Some(not) = parse_not(msg) {
                Message::Notification(not)
            } else {
                Message::InvalidFormat
            }
        }
        _ => Message::InvalidFormat,
    }
}

#[derive(Debug)]
pub struct Request<B> {
    pub msgid: MessageId,
    pub method: Bin<B>,
    pub params: Array<B>,
}

fn parse_req<B: Buf>(mut msg: Array<B>) -> Option<Request<B::Slice>> {
    if let Some(msgid) = msg.next()
        && let Some(msgid) = MessageId::from_obj(msgid)
        && let Some(Object::Str(method)) = msg.next()
        && let Some(Object::Array(params)) = msg.next()
    {
        Some(Request {
            msgid,
            method,
            params,
        })
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Response<B> {
    pub msgid: MessageId,
    pub result: Result<Object<B>, Object<B>>,
}

fn parse_res<B: Buf>(mut msg: Array<B>) -> Option<Response<B::Slice>> {
    if let Some(msgid) = msg.next()
        && let Some(msgid) = MessageId::from_obj(msgid)
        && let Some(error) = msg.next()
        && let Some(result) = msg.next()
    {
        let result = match (result, error) {
            (obj, Object::Nil) => Ok(obj),
            (Object::Nil, obj) => Err(obj),
            _ => return None,
        };

        Some(Response { msgid, result })
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Notification<B> {
    pub method: Bin<B>,
    pub params: Array<B>,
}

fn parse_not<B: Buf>(mut msg: Array<B>) -> Option<Notification<B::Slice>> {
    if let Some(Object::Str(method)) = msg.next()
        && let Some(Object::Array(params)) = msg.next()
    {
        Some(Notification { method, params })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_id_accepts_all_u32_values_encoded_as_i64() {
        use crate::decode::ObjectParser;

        for id in [
            0,
            i64::from(i32::MAX) + 1,
            i64::from(u32::MAX),
            -1,
            i64::from(u32::MAX) + 1,
        ] {
            let mut bytes = [0x94, 0, 0xd3, 0, 0, 0, 0, 0, 0, 0, 0, 0xa1, b'm', 0x90];
            bytes[3..11].copy_from_slice(&id.to_be_bytes());
            let obj = ObjectParser::new(&bytes[..]).next_object().unwrap();
            match (Message::from(obj), u32::try_from(id)) {
                (Message::Request(req), Ok(id)) => {
                    assert_eq!(req.msgid, MessageId(id));
                    assert_eq!(req.method.as_slice(), b"m");
                    assert_eq!(req.params.len(), 0);
                }
                (Message::InvalidFormat, Err(_)) => {}
                _ => panic!("incorrect request message ID"),
            }
        }
    }
}
