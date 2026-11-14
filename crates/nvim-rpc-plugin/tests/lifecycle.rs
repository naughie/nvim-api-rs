use nvim_rpc_plugin::{ConnectionError, Error, Handler, Neovim, api, spawn};

use tokio::io::{AsyncReadExt as _, AsyncWrite, duplex, split};
use tokio::time::{Duration, timeout};

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
