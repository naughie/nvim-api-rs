use nvim_rpc::Receiver;
use nvim_rpc::read::TokioRead;
use nvim_rpc::rpc::Message;
use nvim_rpc_plugin::mp::encode::WriterSyncExt as _;
use nvim_rpc_plugin::mp::encode::primitive::{Nil, Primitive};
use nvim_rpc_plugin::{Bin, ConnectionError, Error, Handler, Neovim, api, mp, spawn};

use tokio::io::{AsyncWrite, AsyncWriteExt as _, BufWriter};
use tokio::sync::{Notify, mpsc};

use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(2), future)
        .await
        .expect("RPC operation timed out")
}

async fn send(writer: &mut (impl AsyncWrite + Unpin), value: impl Primitive) {
    let mut buf = Vec::new();
    buf.write_primitive(value).consume();
    writer.write_all(&buf).await.unwrap();
}

struct Noop;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Noop {
    type Request = ();
    type Response = ();
    type Notification = ();

    async fn on_request(&self, _: &[u8], _: (), _: Neovim<W>) {}

    async fn on_notification(&self, _: &[u8], _: (), _: Neovim<W>) {}
}

struct Arithmetic(mpsc::UnboundedSender<Bin>);

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Arithmetic {
    type Request = (i32, i32, Bin);
    type Response = Result<i32, ()>;
    type Notification = (Bin,);

    async fn on_request(
        &self,
        method: &[u8],
        (lhs, rhs, bin): Self::Request,
        _: Neovim<W>,
    ) -> Self::Response {
        assert_eq!(method, b"add");
        assert_eq!(bin.as_slice(), b"\xffpayload");
        Ok(lhs + rhs)
    }

    async fn on_notification(&self, method: &[u8], (bin,): Self::Notification, _: Neovim<W>) {
        assert_eq!(method, b"retain");
        self.0.send(bin).unwrap();
    }
}

#[tokio::test]
async fn typed_callbacks_retain_bytes_and_reject_bad_request_params() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (_nvim, task) = spawn(reader, writer, Arithmetic(tx));

    send(&mut peer_writer, (2u8, "retain", (&b"\xffkept"[..],))).await;
    let retained = bounded(rx.recv()).await.unwrap();
    send(
        &mut peer_writer,
        (0u8, 41u32, "add", (20i32, 22i32, &b"\xffpayload"[..])),
    )
    .await;
    let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected response");
    };
    assert_eq!(response.msgid.0, 41);
    assert_eq!(response.result.unwrap().to_u8(), Some(42));
    assert_eq!(retained.as_slice(), b"\xffkept");

    send(&mut peer_writer, (0u8, 42u32, "add", (1i32,))).await;
    let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected error response");
    };
    assert_eq!(response.msgid.0, 42);
    assert!(response.result.is_err());
    task.abort();
}

#[tokio::test]
async fn concurrent_typed_requests_match_out_of_order_replies_and_flush() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (nvim, task) = spawn(reader, BufWriter::new(writer), Noop);
    let width_nvim = nvim.clone();
    let width = tokio::spawn(async move {
        let value: i32 = width_nvim.request::<api::NvimStrwidth, _>(("abc",)).await?;
        Ok::<_, Error>(value)
    });
    let line = tokio::spawn(async move {
        let value: String = nvim.request::<api::NvimGetCurrentLine, _>(()).await?;
        Ok::<_, Error>(value)
    });

    let mut requests = Vec::new();
    for _ in 0..2 {
        let Message::Request(request) = bounded(receiver.recv_async()).await.unwrap() else {
            panic!("expected request");
        };
        requests.push((request.msgid.0, request.method.as_slice().to_vec()));
    }
    assert_ne!(requests[0].0, requests[1].0);
    for (id, method) in requests.into_iter().rev() {
        if method == b"nvim_strwidth" {
            send(&mut peer_writer, (1u8, id, Nil, 3i32)).await;
        } else {
            assert_eq!(method, b"nvim_get_current_line");
            send(&mut peer_writer, (1u8, id, Nil, "current line")).await;
        }
    }
    assert_eq!(bounded(width).await.unwrap().unwrap(), 3);
    assert_eq!(bounded(line).await.unwrap().unwrap(), "current line");
    task.abort();
}

struct Ordered {
    started: mpsc::UnboundedSender<i32>,
    finished: mpsc::UnboundedSender<i32>,
    older: Arc<Notify>,
    newer: Arc<Notify>,
}

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Ordered {
    type Request = (i32,);
    type Response = Result<i32, ()>;
    type Notification = ();

    async fn on_request(&self, _: &[u8], (value,): Self::Request, _: Neovim<W>) -> Self::Response {
        self.started.send(value).unwrap();
        if value == 1 {
            self.older.notified().await;
        } else {
            self.newer.notified().await;
        }
        self.finished.send(value).unwrap();
        Ok(value)
    }

    async fn on_notification(&self, _: &[u8], _: (), _: Neovim<W>) {}
}

#[tokio::test]
async fn incoming_replies_follow_reverse_request_order() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (started, mut starts) = mpsc::unbounded_channel();
    let (finished, mut finishes) = mpsc::unbounded_channel();
    let older = Arc::new(Notify::new());
    let newer = Arc::new(Notify::new());
    let (_nvim, task) = spawn(
        reader,
        writer,
        Ordered {
            started,
            finished,
            older: Arc::clone(&older),
            newer: Arc::clone(&newer),
        },
    );
    send(&mut peer_writer, (0u8, 10u32, "ordered", (1i32,))).await;
    assert_eq!(bounded(starts.recv()).await, Some(1));
    send(&mut peer_writer, (0u8, 11u32, "ordered", (2i32,))).await;
    assert_eq!(bounded(starts.recv()).await, Some(2));
    older.notify_one();
    assert_eq!(bounded(finishes.recv()).await, Some(1));
    assert!(
        tokio::time::timeout(Duration::from_millis(20), receiver.recv_async())
            .await
            .is_err()
    );
    newer.notify_one();
    for id in [11, 10] {
        let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
            panic!("expected response");
        };
        assert_eq!(response.msgid.0, id);
        assert!(response.result.is_ok());
    }
    task.abort();
}

#[tokio::test]
async fn cancelling_a_waiter_tolerates_its_late_response() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (nvim, task) = spawn(reader, writer, Noop);
    let first_nvim = nvim.clone();
    let first = tokio::spawn(async move {
        first_nvim
            .request::<api::NvimStrwidth, i32>(("first",))
            .await
    });
    let Message::Request(request) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected request");
    };
    let old_id = request.msgid.0;
    first.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    send(&mut peer_writer, (1u8, old_id, Nil, 5i32)).await;

    let second =
        tokio::spawn(async move { nvim.request::<api::NvimStrwidth, i32>(("second",)).await });
    let Message::Request(request) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected subsequent request");
    };
    assert_ne!(request.msgid.0, old_id);
    send(&mut peer_writer, (1u8, request.msgid.0, Nil, 6i32)).await;
    assert_eq!(bounded(second).await.unwrap().unwrap(), 6);
    task.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn coalesced_invalid_and_valid_requests_keep_reverse_reply_order() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (_nvim, task) = spawn(reader, writer, Noop);
    let mut batch = Vec::new();
    batch
        .write_primitive((0u8, 20u32, "invalid", (1i32,)))
        .consume();
    batch.write_primitive((0u8, 21u32, "noop", ())).consume();

    for _ in 0..100 {
        peer_writer.write_all(&batch).await.unwrap();
        for id in [21, 20] {
            let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
                panic!("expected response");
            };
            assert_eq!(response.msgid.0, id);
            if id == 21 {
                let mp::decode::Object::Array(value) = response.result.unwrap() else {
                    panic!("unit response must be an empty array");
                };
                assert_eq!(value.remaining_len(), 0);
            } else {
                assert!(response.result.is_err());
            }
        }
    }
    task.abort();
}

struct NoopResult;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for NoopResult {
    type Request = ();
    type Response = Result<(), Infallible>;
    type Notification = ();

    async fn on_request(&self, _: &[u8], _: (), _: Neovim<W>) -> Self::Response {
        Ok(())
    }

    async fn on_notification(&self, _: &[u8], _: (), _: Neovim<W>) {}
}

#[tokio::test]
async fn infallible_unit_result_returns_a_successful_empty_array() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (peer_reader, mut peer_writer) = tokio::io::split(peer);
    let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
    let (_nvim, task) = spawn(reader, writer, NoopResult);
    send(&mut peer_writer, (0u8, 1u32, "noop", ())).await;
    let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected response");
    };
    let mp::decode::Object::Array(value) = response.result.unwrap() else {
        panic!("unit response must be an empty array");
    };
    assert_eq!(value.remaining_len(), 0);
    task.abort();
}

#[tokio::test]
async fn eof_wakes_pending_requests_and_closes_retained_clients() {
    let (stream, peer) = tokio::io::duplex(4096);
    let (reader, writer) = tokio::io::split(stream);
    let (nvim, driver) = spawn(reader, writer, Noop);
    let client = nvim.clone();
    let request =
        tokio::spawn(async move { client.request::<api::NvimStrwidth, i32>(("pending",)).await });
    let mut receiver = Receiver::new(TokioRead(peer), [0; 4096]);
    assert!(matches!(
        bounded(receiver.recv_async()).await.unwrap(),
        Message::Request(_)
    ));
    drop(receiver);
    assert!(matches!(
        bounded(request).await.unwrap(),
        Err(Error::Connection(ConnectionError::Closed))
    ));
    bounded(driver).await.unwrap().unwrap();
    assert!(matches!(
        nvim.notify::<api::NvimGetCurrentLine>(()).await,
        Err(Error::Connection(ConnectionError::Closed))
    ));
}

#[tokio::test]
async fn handlers_make_progress_with_an_always_ready_reader() {
    use tokio::io::{AsyncRead, ReadBuf};

    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct Flood {
        first: bool,
        offset: usize,
    }

    impl AsyncRead for Flood {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let bytes: &[u8] = if self.first {
                b"\x94\x00\x01\xa4noop\x90"
            } else {
                b"\x93\x02\xa4noop\x90"
            };
            let n = (bytes.len() - self.offset).min(buf.remaining());
            buf.put_slice(&bytes[self.offset..self.offset + n]);
            self.offset += n;
            if self.offset == bytes.len() {
                self.first = false;
                self.offset = 0;
            }
            Poll::Ready(Ok(()))
        }
    }

    let (writer, peer) = tokio::io::duplex(4096);
    let (_nvim, driver) = spawn(
        Flood {
            first: true,
            offset: 0,
        },
        writer,
        Noop,
    );
    let mut receiver = Receiver::new(TokioRead(peer), [0; 4096]);
    let Message::Response(response) = bounded(receiver.recv_async()).await.unwrap() else {
        panic!("expected handler response during notification flood");
    };
    assert_eq!(response.msgid.0, 1);
    assert!(response.result.is_ok());
    driver.abort();
}
