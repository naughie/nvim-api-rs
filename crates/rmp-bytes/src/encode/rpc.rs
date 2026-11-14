pub use crate::rpc::MessageId;

use super::primitive::{ArrayLike, Primitive, Sealed, State, StrLike};

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

impl<T: Primitive, E: Primitive> Response<T, E> {
    pub fn to_primitive(self) -> impl Primitive<State: State<Item: Send> + Send> + Send
    where
        T: Send,
        T::State: Send,
        <T::State as State>::Item: Send,
        E: Send,
        E::State: Send,
        <E::State as State>::Item: Send,
    {
        use super::primitive::Nil;

        struct Nullable<T>(Option<T>);
        impl<T> Sealed for Nullable<T> {}
        impl<T: Primitive> Primitive for Nullable<T> {
            type State = NullableState<T::State>;
            fn into_state(self) -> Self::State {
                match self.0 {
                    Some(v) => NullableState::Some(v.into_state()),
                    None => NullableState::Null(Nil.into_state()),
                }
            }
        }

        enum NullableItem<T> {
            Some(T),
            Null(<<Nil as Primitive>::State as State>::Item),
        }
        impl<T: AsRef<[u8]>> AsRef<[u8]> for NullableItem<T> {
            fn as_ref(&self) -> &[u8] {
                match self {
                    Self::Some(v) => v.as_ref(),
                    Self::Null(v) => v.as_ref(),
                }
            }
        }

        enum NullableState<T> {
            Some(T),
            Null(<Nil as Primitive>::State),
        }
        impl<T: State> State for NullableState<T> {
            type Item = NullableItem<T::Item>;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::Some(v) => v.next().map(NullableItem::Some),
                    Self::Null(v) => v.next().map(NullableItem::Null),
                }
            }
        }

        let (err, resp) = match self.result {
            Ok(v) => (Nullable(None), Nullable(Some(v))),
            Err(v) => (Nullable(Some(v)), Nullable(None)),
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
