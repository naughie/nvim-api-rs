use nvim_rpc::Receiver;
use nvim_rpc::read::TokioRead;
use nvim_rpc::rpc::Message;
use nvim_rpc_plugin::mp::encode::WriterSyncExt as _;
use nvim_rpc_plugin::mp::encode::primitive::Nil;
use nvim_rpc_plugin::{ConnectionError, Error, Handler, Neovim, api, spawn};

use tokio::io::{AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _, duplex, split};
use tokio::sync::{Notify, mpsc};
use tokio::time::{Duration, timeout};

use std::sync::Arc;

struct Noop;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Noop {
    type Request = ();
    type Response = ();
    type Notification = ();

    async fn on_request(&self, _: &[u8], _: (), _: Neovim<W>) {}

    async fn on_notification(&self, _: &[u8], _: (), _: Neovim<W>) {}
}

#[tokio::test]
async fn aborting_before_the_driver_is_polled_closes_its_client() {
    let (transport, mut peer) = duplex(64);
    let (reader, writer) = split(transport);
    let (nvim, task) = spawn(reader, writer, Noop);
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(matches!(
        nvim.notify::<api::NvimGetCurrentLine>(()).await,
        Err(Error::Connection(ConnectionError::Closed))
    ));
    assert_eq!(peer.read(&mut [0]).await.unwrap(), 0);
}

#[tokio::test]
async fn cancelling_a_partial_write_closes_the_connection() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(1);
        let (reader, writer) = split(transport);
        let (nvim, driver) = spawn(reader, writer, Noop);
        let client = nvim.clone();
        let request =
            tokio::spawn(
                async move { client.request::<api::NvimGetCurrentLine, String>(()).await },
            );
        let mut first = [0];
        peer.read_exact(&mut first).await.unwrap();
        assert_eq!(first, [0x94]);
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());
        assert!(matches!(
            nvim.notify::<api::NvimGetCurrentLine>(()).await,
            Err(Error::Connection(ConnectionError::Interrupted))
        ));
        assert!(matches!(
            driver.await.unwrap(),
            Err(ConnectionError::Interrupted)
        ));
    })
    .await
    .expect("connection did not close after a cancelled write");
}

struct BlockedNotification {
    events: mpsc::UnboundedSender<&'static str>,
    release: Arc<Notify>,
    panic: bool,
}

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for BlockedNotification {
    type Request = ();
    type Response = ();
    type Notification = ();

    async fn on_request(&self, _: &[u8], _: (), _: Neovim<W>) {
        self.events.send("request").unwrap();
    }

    async fn on_notification(&self, method: &[u8], _: (), _: Neovim<W>) {
        struct Guard<'a>(&'a mpsc::UnboundedSender<&'static str>);

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.0.send("dropped").unwrap();
            }
        }

        if method != b"block" {
            self.events.send("notification").unwrap();
            return;
        }

        let _guard = Guard(&self.events);
        self.events.send("started").unwrap();
        self.release.notified().await;
        assert!(!self.panic, "notification panic");
    }
}

enum Stop {
    Abort,
    Eof,
    Panic,
}

async fn stop_blocked_notification(stop: Stop) {
    timeout(Duration::from_secs(2), async {
        let (transport, peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (peer_reader, mut peer_writer) = split(peer);
        let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
        let (events, mut received) = mpsc::unbounded_channel();
        let release = Arc::new(Notify::new());
        let (nvim, driver) = spawn(
            reader,
            writer,
            BlockedNotification {
                events,
                release: Arc::clone(&release),
                panic: matches!(stop, Stop::Panic),
            },
        );

        let client = nvim.clone();
        let sync =
            tokio::spawn(async move { client.request::<api::NvimStrwidth, i32>(("sync",)).await });
        let Message::Request(request) = receiver.recv_async().await.unwrap() else {
            panic!("expected synchronization request");
        };
        let mut batch = Vec::new();
        batch.write_primitive((2u8, "block", ())).consume();
        batch.write_primitive((0u8, 41u32, "queued", ())).consume();
        batch.write_primitive((2u8, "queued", ())).consume();
        batch
            .write_primitive((1u8, request.msgid.0, Nil, 4i32))
            .consume();
        peer_writer.write_all(&batch).await.unwrap();
        // Routing the final response proves that both later callbacks were received.
        assert_eq!(sync.await.unwrap().unwrap(), 4);
        assert_eq!(received.recv().await, Some("started"));

        match stop {
            Stop::Abort => {
                driver.abort();
                assert!(driver.await.unwrap_err().is_cancelled());
            }
            Stop::Eof => {
                peer_writer.shutdown().await.unwrap();
                driver.await.unwrap().unwrap();
            }
            Stop::Panic => {
                release.notify_one();
                let Err(ConnectionError::Task(error)) = driver.await.unwrap() else {
                    panic!("expected notification task failure");
                };
                assert!(error.is_panic());
            }
        }
        assert_eq!(received.recv().await, Some("dropped"));
        assert_eq!(received.recv().await, None);
        match (stop, nvim.notify::<api::NvimGetCurrentLine>(()).await) {
            (Stop::Panic, Err(Error::Connection(ConnectionError::Task(error)))) => {
                assert!(error.is_panic());
            }
            (Stop::Abort | Stop::Eof, Err(Error::Connection(ConnectionError::Closed))) => {}
            (_, result) => panic!("unexpected connection state: {result:?}"),
        }
    })
    .await
    .expect("notification shutdown timed out");
}

#[tokio::test]
async fn aborting_cancels_active_notifications_and_discards_queued_callbacks() {
    stop_blocked_notification(Stop::Abort).await;
}

#[tokio::test]
async fn eof_cancels_active_notifications_and_discards_queued_callbacks() {
    stop_blocked_notification(Stop::Eof).await;
}

#[tokio::test]
async fn notification_panics_close_the_connection_without_dispatching_queued_callbacks() {
    stop_blocked_notification(Stop::Panic).await;
}
