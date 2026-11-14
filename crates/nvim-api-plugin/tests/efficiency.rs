use nvim_api_plugin::{Handler, Neovim, api, spawn};

use tokio::io::{AsyncWrite, duplex};

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

struct Noop;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Noop {
    type Request = ();
    type Response = ();
    type Notification = ();

    async fn on_request(&self, _: &[u8], _: (), _: Neovim<W>) {}
    async fn on_notification(&self, _: &[u8], _: (), _: Neovim<W>) {}
}

struct CountWrites(Arc<AtomicUsize>);

impl AsyncWrite for CountWrites {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Poll::Ready(Ok(bytes.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn combines_small_message_chunks_before_transport_writes() {
    let writes = Arc::new(AtomicUsize::new(0));
    let (from_neovim, _peer) = duplex(64);
    let (nvim, driver) = spawn(from_neovim, CountWrites(Arc::clone(&writes)), Noop);

    for expected in 1..=2 {
        nvim.notify::<api::NvimSetCurrentLine>(("hello",))
            .await
            .unwrap();
        assert_eq!(writes.load(Ordering::Relaxed), expected);
    }
    driver.abort();
}
