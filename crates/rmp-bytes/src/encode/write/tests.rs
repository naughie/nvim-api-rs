use super::*;

use rmpv::Value;

use std::io::Cursor;

fn primitives() -> (
    (impl Primitive + Clone, Value),
    (impl Primitive + Clone, Value),
    (impl Primitive + Clone, Value),
) {
    use crate::encode::primitive::Array;

    (
        (true, Value::Boolean(true)),
        ("the quick brown fox", Value::from("the quick brown fox")),
        (
            Array([0u8, 1u8, 2u8]),
            Value::Array(vec![Value::from(0u8), Value::from(1u8), Value::from(2u8)]),
        ),
    )
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut wtr = Cursor::new(&mut buf);
    rmpv::encode::write_value(&mut wtr, value).unwrap();
    buf
}

#[test]
fn buf_sync() {
    fn test_write((prim, value): &(impl Primitive + Clone, Value)) {
        let mut our_buf = Vec::new();
        <_ as WriterSyncExt>::write_primitive(&mut our_buf, prim.clone()).consume();

        assert_eq!(our_buf, encode_value(value));
    }

    #[cfg(feature = "bytes")]
    fn test_write_bytes((prim, value): &(impl Primitive + Clone, Value)) {
        use bytes::BytesMut;

        let mut our_buf = BufWriter::new(BytesMut::new());
        <_ as WriterSyncExt>::write_primitive(&mut our_buf, prim.clone()).consume();

        assert_eq!(our_buf.get_ref(), &encode_value(value));
    }

    let (test1, test2, test3) = primitives();
    test_write(&test1);
    test_write(&test2);
    test_write(&test3);

    #[cfg(feature = "bytes")]
    {
        test_write_bytes(&test1);
        test_write_bytes(&test2);
        test_write_bytes(&test3);
    }
}

#[tokio::test]
async fn buf_async() {
    async fn test_write((prim, value): &(impl Primitive + Clone, Value)) {
        let mut our_buf = Vec::new();
        <_ as WriterAsyncExt>::write_primitive(&mut our_buf, prim.clone())
            .consume_async()
            .await;

        assert_eq!(our_buf, encode_value(value));
    }

    #[cfg(feature = "bytes")]
    async fn test_write_bytes((prim, value): &(impl Primitive + Clone, Value)) {
        use bytes::BytesMut;

        let mut our_buf = BufWriter::new(BytesMut::new());
        <_ as WriterAsyncExt>::write_primitive(&mut our_buf, prim.clone())
            .consume_async()
            .await;

        assert_eq!(our_buf.get_ref(), &encode_value(value));
    }

    let (test1, test2, test3) = primitives();
    test_write(&test1).await;
    test_write(&test2).await;
    test_write(&test3).await;

    #[cfg(feature = "bytes")]
    {
        test_write_bytes(&test1).await;
        test_write_bytes(&test2).await;
        test_write_bytes(&test3).await;
    }
}

#[tokio::test]
async fn buffer_writes_start_when_polled() {
    let mut bytes = Vec::new();
    drop(WriterAsync::write_all(&mut bytes, b"discarded"));
    assert!(bytes.is_empty());
    WriterAsync::write_all(&mut bytes, b"written").await;
    assert_eq!(bytes, b"written");

    #[cfg(feature = "bytes")]
    {
        let mut bytes = BufWriter::new(bytes::BytesMut::new());
        drop(WriterAsync::write_all(&mut bytes, b"discarded"));
        assert!(bytes.get_ref().is_empty());
        WriterAsync::write_all(&mut bytes, b"written").await;
        assert_eq!(bytes.get_ref().as_ref(), b"written");
    }
}

#[test]
fn std_sync() {
    fn test_write((prim, value): &(impl Primitive + Clone, Value)) {
        let mut our_buf = StdWrite::new(Vec::new());
        let res = <_ as WriterSyncExt>::write_primitive(&mut our_buf, prim.clone()).consume();

        assert!(res.is_ok());
        assert_eq!(our_buf.get_ref(), &encode_value(value));
    }

    let (test1, test2, test3) = primitives();
    test_write(&test1);
    test_write(&test2);
    test_write(&test3);
}

#[tokio::test]
async fn async_chunks_can_be_borrowed_sent_and_consumed_incrementally() {
    fn assert_send(_: impl Future + Send) {}

    let mut bytes = Vec::new();
    let mut chunks = WriterAsyncExt::write_primitive(&mut bytes, (1u8, "x"));
    assert_send(chunks.next_chunk_async());
    assert!(chunks.next_chunk_async().await.is_some());
    chunks.consume_async().await;
    assert_eq!(bytes, b"\x92\x01\xa1x");

    let mut bytes = Vec::new();
    {
        let mut chunks = WriterAsyncExt::write_primitive(&mut bytes, (1u8, "x"));
        while chunks.next_chunk_async().await.is_some() {}
        assert!(chunks.next_chunk_async().await.is_none());
    }
    assert_eq!(bytes, b"\x92\x01\xa1x");
}

#[cfg(feature = "tokio-write")]
#[tokio::test]
async fn tokio_async() {
    async fn test_write((prim, value): &(impl Primitive + Clone, Value)) {
        let mut our_buf = TokioWrite::new(Vec::new());
        let res = <_ as WriterAsyncExt>::write_primitive(&mut our_buf, prim.clone())
            .consume_async()
            .await;

        assert!(res.is_ok());
        assert_eq!(our_buf.get_ref(), &encode_value(value));
    }

    let (test1, test2, test3) = primitives();
    test_write(&test1).await;
    test_write(&test2).await;
    test_write(&test3).await;
}

#[cfg(feature = "futures-write")]
#[tokio::test]
async fn futures_async() {
    async fn test_write((prim, value): &(impl Primitive + Clone, Value)) {
        let mut our_buf = FuturesWrite::new(Vec::new());
        let res = <_ as WriterAsyncExt>::write_primitive(&mut our_buf, prim.clone())
            .consume_async()
            .await;

        assert!(res.is_ok());
        assert_eq!(our_buf.get_ref(), &encode_value(value));
    }

    let (test1, test2, test3) = primitives();
    test_write(&test1).await;
    test_write(&test2).await;
    test_write(&test3).await;
}
