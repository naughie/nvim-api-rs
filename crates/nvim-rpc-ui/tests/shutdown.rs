use nvim_rpc::mp::encode::{TokioWrite, WriterSyncExt as _};
use nvim_rpc::read::TokioRead;
use nvim_rpc::{Neovim, Receiver, api, rpc::Message};

use nvim_rpc_ui::{ConnectionOptions, connect as connect_with_handler};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt as _, ReadBuf, duplex, split};
use tokio::time::{Duration, timeout};

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

struct Tracked<T> {
    inner: T,
    dropped: Arc<AtomicBool>,
}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Tracked<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Tracked<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

use nvim_rpc_plugin::ConnectionError;

fn connect<R, W>(
    reader: R,
    writer: W,
    options: ConnectionOptions,
) -> Result<nvim_rpc_ui::Connection<W>, nvim_rpc_ui::Error>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    connect_with_handler(reader, writer, nvim_rpc_ui::NoHandler, options)
}

#[test]
fn connect_without_runtime_returns_error() {
    assert!(matches!(
        connect(
            tokio::io::empty(),
            tokio::io::sink(),
            ConnectionOptions::default()
        ),
        Err(nvim_rpc_ui::Error::Connection(
            ConnectionError::InvalidOptions(_)
        ))
    ));
}

#[tokio::test]
async fn cancelled_cleanup_wait_retries_after_blocked_writer_is_released() {
    struct PendingWriter(Arc<tokio::sync::Notify>);

    impl AsyncWrite for PendingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.0.notify_one();
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let (reader, _remote) = duplex(64);
    let entered = Arc::new(tokio::sync::Notify::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let mut connection = connect(
        reader,
        Tracked {
            inner: PendingWriter(Arc::clone(&entered)),
            dropped: Arc::clone(&dropped),
        },
        ConnectionOptions::default(),
    )
    .unwrap();
    let client = connection.client.clone();
    let mut request = Box::pin(client.nvim().request::<api::NvimEval, i64>(("1",)));
    tokio::select! {
        biased;
        result = &mut request => panic!("blocked write completed: {result:?}"),
        () = entered.notified() => {},
    }
    assert!(!dropped.load(Ordering::SeqCst));
    connection.stop();
    assert!(
        timeout(Duration::from_millis(20), connection.wait())
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    assert!(
        timeout(Duration::from_secs(2), request)
            .await
            .unwrap()
            .is_err()
    );
    timeout(Duration::from_secs(2), connection.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn cancelled_connection_wait_can_be_retried_and_releases_transport() {
    let (local, _remote) = duplex(64);
    let (reader, writer) = split(local);
    let reader_dropped = Arc::new(AtomicBool::new(false));
    let writer_dropped = Arc::new(AtomicBool::new(false));
    let mut connection = connect(
        Tracked {
            inner: reader,
            dropped: Arc::clone(&reader_dropped),
        },
        Tracked {
            inner: writer,
            dropped: Arc::clone(&writer_dropped),
        },
        ConnectionOptions::default(),
    )
    .unwrap();

    tokio::select! {
        biased;
        result = connection.wait() => panic!("open transport completed: {result:?}"),
        () = tokio::task::yield_now() => {},
    }
    assert!(!reader_dropped.load(Ordering::SeqCst));
    assert!(!writer_dropped.load(Ordering::SeqCst));

    connection.stop();
    timeout(Duration::from_secs(2), connection.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(reader_dropped.load(Ordering::SeqCst));
    assert!(writer_dropped.load(Ordering::SeqCst));
    connection.wait().await.unwrap();
    connection.close_and_wait().await.unwrap();
    assert!(connection.frames.recv().await.is_none());
}

#[tokio::test]
async fn eof_finishes_without_waiting_for_presentation_and_retains_latest_frame() {
    let (local, mut remote) = duplex(65536);
    let (reader, writer) = split(local);
    let mut connection = connect(
        reader,
        writer,
        ConnectionOptions {
            ..ConnectionOptions::default()
        },
    )
    .unwrap();
    let mut bytes = Vec::new();
    bytes
        .write_primitive(
            nvim_rpc::mp::encode::rpc::Notification {
                method: "redraw",
                params: (("grid_resize", (1, 2, 2)), ("flush", (), ())),
            }
            .to_primitive(),
        )
        .consume();
    remote.write_all(&bytes).await.unwrap();
    remote.shutdown().await.unwrap();

    timeout(Duration::from_secs(2), connection.wait())
        .await
        .unwrap()
        .unwrap();
    connection.close_and_wait().await.unwrap();
    assert_eq!(connection.frames.recv().await.unwrap().unwrap().sequence, 2);
    assert!(connection.frames.recv().await.is_none());
    connection.wait().await.unwrap();
}

#[tokio::test]
async fn rpc_failure_is_preserved_across_repeated_cleanup_waits() {
    let (local, mut remote) = duplex(65536);
    let (reader, writer) = split(local);
    let mut connection = connect(
        reader,
        writer,
        ConnectionOptions {
            ..ConnectionOptions::default()
        },
    )
    .unwrap();
    let mut bytes = Vec::new();
    bytes
        .write_primitive(
            nvim_rpc::mp::encode::rpc::Notification {
                method: "redraw",
                params: (("flush", ()),),
            }
            .to_primitive(),
        )
        .consume();
    remote.write_all(&bytes).await.unwrap();
    assert_eq!(connection.frames.recv().await.unwrap().unwrap().sequence, 1);
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, 42u8, ())).consume();
    remote.write_all(&bytes).await.unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(2), connection.wait())
            .await
            .unwrap(),
        Err(ConnectionError::InvalidFormat)
    ));

    assert!(matches!(
        timeout(Duration::from_secs(2), connection.close_and_wait())
            .await
            .unwrap(),
        Err(ConnectionError::InvalidFormat)
    ));
    assert!(matches!(
        connection.wait().await,
        Err(ConnectionError::InvalidFormat)
    ));
    assert!(connection.frames.recv().await.is_none());
    assert!(matches!(
        connection
            .client
            .nvim()
            .request::<api::NvimEval, i64>(("1",))
            .await,
        Err(nvim_rpc_plugin::Error::Connection(
            ConnectionError::InvalidFormat
        ))
    ));
}

#[tokio::test]
async fn detach_accepts_extended_success_result_and_keeps_rpc_connected() {
    let (local, remote) = duplex(65536);
    let (reader, writer) = split(local);
    let mut connection = connect(reader, writer, ConnectionOptions::default()).unwrap();
    let (reader, writer) = split(remote);
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(reader), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(writer));
        for (method, result) in [(b"nvim_ui_detach".as_slice(), 42), (b"nvim_eval", 73)] {
            let Message::Request(request) = receiver.recv_async().await.unwrap() else {
                panic!("request expected");
            };
            assert_eq!(request.method.as_slice(), method);
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(result),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        }
    });

    timeout(Duration::from_secs(2), connection.detach_ui())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        timeout(
            Duration::from_secs(2),
            connection
                .client
                .nvim()
                .request::<api::NvimEval, i64>(("73",))
        )
        .await
        .unwrap()
        .unwrap(),
        73
    );
    peer.await.unwrap();
    connection.close_and_wait().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn connection_shutdown_does_not_terminate_external_process() {
    use std::process::Stdio;

    let mut child = tokio::process::Command::new("sleep")
        .arg("60")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut connection = connect(
        child.stdout.take().unwrap(),
        child.stdin.take().unwrap(),
        ConnectionOptions::default(),
    )
    .unwrap();
    timeout(Duration::from_secs(2), connection.close_and_wait())
        .await
        .unwrap()
        .unwrap();
    assert!(child.try_wait().unwrap().is_none());
    drop(connection);
    assert!(child.try_wait().unwrap().is_none());
    timeout(Duration::from_secs(2), child.kill())
        .await
        .unwrap()
        .unwrap();
}
