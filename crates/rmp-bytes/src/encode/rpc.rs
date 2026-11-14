pub use crate::rpc::MessageId;

use super::primitive::{ArrayLike, Nil, Primitive, State, StrLike};
use super::{Encode, Untagged};

#[derive(Debug)]
pub struct Request<M, P> {
    pub msgid: MessageId,
    pub method: M,
    pub params: P,
}

#[derive(Debug)]
pub struct Response<T, E> {
    pub msgid: MessageId,
    pub result: Result<T, E>,
}

#[derive(Debug)]
pub struct Notification<M, P> {
    pub method: M,
    pub params: P,
}

impl<M: StrLike, P: ArrayLike> Request<M, P> {
    pub fn to_primitive(self) -> impl Primitive<State: State<Item: Send> + Send> + Send
    where
        M: Send,
        M::State: Send,
        <M::State as State>::Item: Send,
        P: Send,
        P::State: Send,
        <P::State as State>::Item: Send,
    {
        (0u8, self.msgid.0, self.method, self.params)
    }
}

impl<T: Encode, E: Encode> Response<T, E> {
    pub fn to_primitive(self) -> impl Primitive<State: State<Item: Send> + Send> + Send
    where
        T: Send,
        T::Encoder: Send,
        <T::Encoder as State>::Item: Send,
        E: Send,
        E::Encoder: Send,
        <E::Encoder as State>::Item: Send,
    {
        let (err, resp): (Untagged<E, Nil>, Untagged<T, Nil>) = match self.result {
            Ok(v) => (Untagged::Right(Nil), Untagged::Left(v)),
            Err(v) => (Untagged::Left(v), Untagged::Right(Nil)),
        };
        (1u8, self.msgid.0, err, resp)
    }
}

impl<M: StrLike, P: ArrayLike> Notification<M, P> {
    pub fn to_primitive(self) -> impl Primitive<State: State<Item: Send> + Send> + Send
    where
        M: Send,
        M::State: Send,
        <M::State as State>::Item: Send,
        P: Send,
        P::State: Send,
        <P::State as State>::Item: Send,
    {
        (2u8, self.method, self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{WriterAsyncExt, WriterSyncExt};

    #[test]
    fn response_fields_follow_rpc_layout() {
        let mut bytes = Vec::new();
        WriterSyncExt::write_primitive(
            &mut bytes,
            Response {
                msgid: MessageId(7),
                result: Ok::<_, &str>(42u8),
            }
            .to_primitive(),
        )
        .consume();
        assert_eq!(bytes, b"\x94\x01\x07\xc0\x2a");

        bytes.clear();
        WriterSyncExt::write_primitive(
            &mut bytes,
            Response {
                msgid: MessageId(7),
                result: Err::<u8, _>("bad"),
            }
            .to_primitive(),
        )
        .consume();
        assert_eq!(bytes, b"\x94\x01\x07\xa3bad\xc0");
    }

    #[tokio::test]
    async fn asynchronous_response_fields_follow_rpc_layout() {
        let mut bytes = Vec::new();
        WriterAsyncExt::write_primitive(
            &mut bytes,
            Response {
                msgid: MessageId(7),
                result: Ok::<_, &str>(42u8),
            }
            .to_primitive(),
        )
        .consume_async()
        .await;
        assert_eq!(bytes, b"\x94\x01\x07\xc0\x2a");

        bytes.clear();
        WriterAsyncExt::write_primitive(
            &mut bytes,
            Response {
                msgid: MessageId(7),
                result: Err::<u8, _>("bad"),
            }
            .to_primitive(),
        )
        .consume_async()
        .await;
        assert_eq!(bytes, b"\x94\x01\x07\xa3bad\xc0");
    }
}
