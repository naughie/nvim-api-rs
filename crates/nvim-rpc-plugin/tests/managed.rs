use nvim_rpc::Receiver;
use nvim_rpc::read::TokioRead;
use nvim_rpc::rpc::Message;
use nvim_rpc_plugin::mp::encode::WriterSyncExt as _;
use nvim_rpc_plugin::mp::encode::primitive::Nil;
use nvim_rpc_plugin::{
    Array, ConnectionError, ConnectionOptions, Error, Handler, Neovim, NoHandler, api, connect,
};

use tokio::io::{AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, duplex, split};
use tokio::sync::{Notify, mpsc};
use tokio::time::{Duration, timeout};

use std::sync::Arc;

#[test]
fn rejects_invalid_options_and_missing_runtime() {
    let result = connect(
        tokio::io::empty(),
        tokio::io::sink(),
        NoHandler,
        ConnectionOptions {
            max_rpc_bytes: 0,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(ConnectionError::InvalidOptions(_))));
    let result = connect(
        tokio::io::empty(),
        tokio::io::sink(),
        NoHandler,
        ConnectionOptions::default(),
    );
    assert!(matches!(result, Err(ConnectionError::InvalidOptions(_))));
}

#[tokio::test]
async fn dropping_before_driver_poll_closes_all_client_clones() {
    let (transport, mut peer) = duplex(64);
    let (reader, writer) = split(transport);
    let connection = connect(reader, writer, NoHandler, ConnectionOptions::default()).unwrap();
    let client = connection.client.clone();
    drop(connection);
    assert!(matches!(
        client.notify::<api::NvimGetCurrentLine>(()).await,
        Err(Error::Connection(ConnectionError::Closed))
    ));
    assert_eq!(peer.read(&mut [0]).await.unwrap(), 0);
}

#[tokio::test]
async fn cancelled_wait_remains_repeatable_while_send_holds_writer() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(1);
        let (reader, writer) = split(transport);
        let mut connection =
            connect(reader, writer, NoHandler, ConnectionOptions::default()).unwrap();
        let client = connection.client.clone();
        let mut sending = Box::pin(client.notify::<api::NvimSetCurrentLine>(("waiting",)));
        let mut first = [0];
        tokio::select! {
            result = &mut sending => panic!("write unexpectedly completed: {result:?}"),
            result = peer.read_exact(&mut first) => { result.unwrap(); }
        }
        assert_eq!(first, [0x93]);
        connection.stop();
        assert!(
            timeout(Duration::from_millis(10), connection.wait())
                .await
                .is_err()
        );
        assert!(connection.is_finished());
        drop(sending);
        connection.wait().await.unwrap();
        connection.wait().await.unwrap();
        assert_eq!(peer.read(&mut [0]).await.unwrap(), 0);
    })
    .await
    .unwrap();
}

struct Ordered(mpsc::UnboundedSender<&'static str>);

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Ordered {
    type Request = Array;
    type Response = ();
    type Notification = Array;

    async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}

    async fn on_notification(&self, method: &[u8], _: Array, nvim: Neovim<W>) {
        if method == b"first" {
            assert!(matches!(
                nvim.request::<api::NvimGetCurrentLine, String>(()).await,
                Err(Error::Connection(ConnectionError::Closed))
            ));
            self.0.send("first").unwrap();
        } else {
            self.0.send("second").unwrap();
        }
    }
}

#[tokio::test]
async fn clean_eof_wakes_nested_calls_and_drains_received_notifications() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (events, mut received) = mpsc::unbounded_channel();
        let mut connection = connect(
            reader,
            writer,
            Ordered(events),
            ConnectionOptions::default(),
        )
        .unwrap();
        let mut bytes = Vec::new();
        bytes.write_primitive((2u8, "first", ())).consume();
        bytes.write_primitive((2u8, "second", ())).consume();
        peer.write_all(&bytes).await.unwrap();
        peer.shutdown().await.unwrap();
        connection.wait().await.unwrap();
        connection.wait().await.unwrap();
        assert_eq!(received.recv().await, Some("first"));
        assert_eq!(received.recv().await, Some("second"));
        assert_eq!(received.recv().await, None);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn frame_byte_bound_accepts_exact_length_and_rejects_longer_frames() {
    for (max_rpc_bytes, accepts) in [(4, true), (3, false)] {
        let (transport, mut peer) = duplex(64);
        let (reader, writer) = split(transport);
        let mut connection = connect(
            reader,
            writer,
            NoHandler,
            ConnectionOptions {
                max_rpc_bytes,
                ..Default::default()
            },
        )
        .unwrap();
        peer.write_all(b"\x93\x02\xa0\x90").await.unwrap();
        peer.shutdown().await.unwrap();
        let result = timeout(Duration::from_secs(2), connection.wait())
            .await
            .unwrap();
        if accepts {
            result.unwrap();
        } else {
            assert!(matches!(
                result,
                Err(ConnectionError::Limit("RPC frame bytes"))
            ));
            assert!(matches!(
                connection.wait().await,
                Err(ConnectionError::Limit("RPC frame bytes"))
            ));
        }
    }
}

#[tokio::test]
async fn request_bound_retains_cancelled_waiters_until_late_response() {
    timeout(Duration::from_secs(2), async {
        let (transport, peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (events, mut received) = mpsc::unbounded_channel();
        let mut connection = connect(
            reader,
            writer,
            Ordered(events),
            ConnectionOptions {
                max_pending_requests: 1,
                ..Default::default()
            },
        )
        .unwrap();
        let (reader, mut writer) = split(peer);
        let mut receiver = Receiver::new(TokioRead(reader), [0; 4096]);
        let client = connection.client.clone();
        let request =
            tokio::spawn(
                async move { client.request::<api::NvimGetCurrentLine, String>(()).await },
            );
        let Message::Request(first) = receiver.recv_async().await.unwrap() else {
            panic!("expected initial request");
        };
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());
        assert!(matches!(
            connection
                .client
                .request::<api::NvimGetCurrentLine, String>(())
                .await,
            Err(Error::Connection(ConnectionError::Limit(
                "pending RPC requests"
            )))
        ));
        let mut bytes = Vec::new();
        bytes
            .write_primitive((1u8, first.msgid.0, Nil, "late"))
            .consume();
        // A following notification provides an ordering barrier for routing.
        bytes.write_primitive((2u8, "barrier", ())).consume();
        writer.write_all(&bytes).await.unwrap();
        assert_eq!(received.recv().await, Some("second"));
        let client = connection.client.clone();
        let request =
            tokio::spawn(
                async move { client.request::<api::NvimGetCurrentLine, String>(()).await },
            );
        let Message::Request(next) = receiver.recv_async().await.unwrap() else {
            panic!("expected next request");
        };
        let mut bytes = Vec::new();
        bytes
            .write_primitive((1u8, next.msgid.0, Nil, "ready"))
            .consume();
        writer.write_all(&bytes).await.unwrap();
        assert_eq!(request.await.unwrap().unwrap(), "ready");
        connection.close_and_wait().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn bounds_incoming_reply_stack_before_dispatching_more_requests() {
    let (transport, mut peer) = duplex(4096);
    let (reader, writer) = split(transport);
    let mut connection = connect(
        reader,
        writer,
        NoHandler,
        ConnectionOptions {
            max_pending_replies: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut bytes = Vec::new();
    for id in [1u32, 2] {
        bytes.write_primitive((0u8, id, "request", ())).consume();
    }
    peer.write_all(&bytes).await.unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(2), connection.wait())
            .await
            .unwrap(),
        Err(ConnectionError::Limit("pending RPC replies"))
    ));
}

#[tokio::test]
async fn bounds_callbacks_while_a_notification_is_blocked() {
    struct Block(Arc<Notify>);

    impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Block {
        type Request = Array;
        type Response = ();
        type Notification = Array;

        async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}
        async fn on_notification(&self, _: &[u8], _: Array, _: Neovim<W>) {
            self.0.notified().await;
        }
    }

    let (transport, mut peer) = duplex(4096);
    let (reader, writer) = split(transport);
    let mut connection = connect(
        reader,
        writer,
        Block(Arc::new(Notify::new())),
        ConnectionOptions {
            max_pending_callbacks: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut bytes = Vec::new();
    for _ in 0..3 {
        bytes.write_primitive((2u8, "block", ())).consume();
    }
    peer.write_all(&bytes).await.unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(2), connection.wait())
            .await
            .unwrap(),
        Err(ConnectionError::Limit("pending RPC callbacks"))
    ));
}

#[tokio::test]
async fn stop_during_eof_drain_joins_handler_cleanup_and_discards_queued_work() {
    struct Block(mpsc::UnboundedSender<&'static str>);

    impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Block {
        type Request = Array;
        type Response = ();
        type Notification = Array;

        async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}

        async fn on_notification(&self, method: &[u8], _: Array, nvim: Neovim<W>) {
            struct Guard<'a>(&'a mpsc::UnboundedSender<&'static str>);
            impl Drop for Guard<'_> {
                fn drop(&mut self) {
                    self.0.send("dropped").unwrap();
                }
            }

            if method != b"first" {
                self.0.send("unexpected callback").unwrap();
                return;
            }
            let _guard = Guard(&self.0);
            assert!(matches!(
                nvim.request::<api::NvimGetCurrentLine, String>(()).await,
                Err(Error::Connection(ConnectionError::Closed))
            ));
            self.0.send("draining").unwrap();
            std::future::pending::<()>().await;
        }
    }

    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (events, mut received) = mpsc::unbounded_channel();
        let mut connection =
            connect(reader, writer, Block(events), ConnectionOptions::default()).unwrap();
        let mut bytes = Vec::new();
        bytes.write_primitive((2u8, "first", ())).consume();
        bytes.write_primitive((2u8, "queued", ())).consume();
        peer.write_all(&bytes).await.unwrap();
        peer.shutdown().await.unwrap();
        assert_eq!(received.recv().await, Some("draining"));
        connection.stop();
        connection.wait().await.unwrap();
        assert_eq!(received.recv().await, Some("dropped"));
        assert_eq!(received.recv().await, None);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn handler_failure_during_eof_drain_remains_the_repeatable_result() {
    struct Panics;
    impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Panics {
        type Request = Array;
        type Response = ();
        type Notification = Array;

        async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}
        async fn on_notification(&self, _: &[u8], _: Array, nvim: Neovim<W>) {
            assert!(matches!(
                nvim.request::<api::NvimGetCurrentLine, String>(()).await,
                Err(Error::Connection(ConnectionError::Closed))
            ));
            panic!("handler failed during EOF drain");
        }
    }

    let (transport, mut peer) = duplex(4096);
    let (reader, writer) = split(transport);
    let mut connection = connect(reader, writer, Panics, ConnectionOptions::default()).unwrap();
    peer.write_all(b"\x93\x02\xa0\x90").await.unwrap();
    peer.shutdown().await.unwrap();
    let Err(ConnectionError::Task(first)) = timeout(Duration::from_secs(2), connection.wait())
        .await
        .unwrap()
    else {
        panic!("expected handler failure");
    };
    let Err(ConnectionError::Task(second)) = connection.wait().await else {
        panic!("expected retained handler failure");
    };
    assert!(first.is_panic());
    assert!(Arc::ptr_eq(&first, &second));
}

#[tokio::test]
async fn handler_close_during_eof_drain_discards_later_callbacks() {
    struct Stopper(mpsc::UnboundedSender<&'static str>);
    impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Stopper {
        type Request = Array;
        type Response = ();
        type Notification = Array;

        async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {
            self.0.send("unexpected request").unwrap();
        }

        async fn on_notification(&self, method: &[u8], _: Array, nvim: Neovim<W>) {
            if method != b"first" {
                self.0.send("unexpected notification").unwrap();
                return;
            }
            // The unanswered nested call completes only once EOF closes RPC.
            assert!(matches!(
                nvim.request::<api::NvimGetCurrentLine, String>(()).await,
                Err(Error::Connection(ConnectionError::Closed))
            ));
            nvim.close();
            self.0.send("stopped").unwrap();
        }
    }

    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (events, mut received) = mpsc::unbounded_channel();
        let mut connection = connect(
            reader,
            writer,
            Stopper(events),
            ConnectionOptions::default(),
        )
        .unwrap();
        let mut bytes = Vec::new();
        bytes.write_primitive((2u8, "first", ())).consume();
        bytes.write_primitive((2u8, "queued", ())).consume();
        bytes.write_primitive((0u8, 17u32, "queued", ())).consume();
        peer.write_all(&bytes).await.unwrap();
        peer.shutdown().await.unwrap();
        assert!(matches!(
            connection.wait().await,
            Err(ConnectionError::Closed)
        ));
        assert_eq!(received.recv().await, Some("stopped"));
        assert_eq!(received.recv().await, None);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn no_handler_rejects_incoming_requests_in_reverse_order() {
    let (transport, peer) = duplex(4096);
    let (reader, writer) = split(transport);
    let mut connection = connect(reader, writer, NoHandler, ConnectionOptions::default()).unwrap();
    let (reader, mut writer) = split(peer);
    let mut receiver = Receiver::new(TokioRead(reader), [0; 4096]);
    let mut bytes = Vec::new();
    for id in [1u32, 2] {
        bytes.write_primitive((0u8, id, "request", ())).consume();
    }
    writer.write_all(&bytes).await.unwrap();
    for id in [2u32, 1] {
        let Message::Response(response) = timeout(Duration::from_secs(2), receiver.recv_async())
            .await
            .unwrap()
            .unwrap()
        else {
            panic!("expected method-error response");
        };
        assert_eq!(response.msgid.0, id);
        assert!(response.result.is_err());
    }
    connection.close_and_wait().await.unwrap();
}

#[tokio::test]
async fn explicit_stop_preserves_an_existing_writer_failure() {
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct Fails;
    impl AsyncWrite for Fails {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "test writer",
            )))
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let (reader, _peer) = duplex(64);
    let mut connection = connect(reader, Fails, NoHandler, ConnectionOptions::default()).unwrap();
    let Err(Error::Connection(ConnectionError::Io(first))) = connection
        .client
        .notify::<api::NvimSetCurrentLine>(("line",))
        .await
    else {
        panic!("expected writer failure");
    };
    connection.stop();
    let Err(ConnectionError::Io(second)) = timeout(Duration::from_secs(2), connection.wait())
        .await
        .unwrap()
    else {
        panic!("expected original connection failure");
    };
    assert_eq!(first.kind(), io::ErrorKind::BrokenPipe);
    assert!(Arc::ptr_eq(&first, &second));
}
