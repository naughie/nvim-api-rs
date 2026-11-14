use nvim_api_plugin::{Array, Bin, Bytes, Decode, Handler, Neovim, api, spawn, sys};

use tokio::io::AsyncWrite;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

use std::process::Stdio;

struct Plugin(mpsc::UnboundedSender<Bytes>);

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Plugin {
    type Request = (i32, i32, Bin);
    type Response = Result<i32, ()>;
    type Notification = (Bin,);

    async fn on_request(
        &self,
        method: &[u8],
        (lhs, rhs, text): Self::Request,
        nvim: Neovim<W>,
    ) -> Self::Response {
        assert_eq!(method, b"add");
        assert_eq!(text.as_slice(), b"argument");
        let line: Bin = nvim
            .request::<api::NvimGetCurrentLine, _>(())
            .await
            .unwrap();
        assert_eq!(line.as_slice(), b"before");
        lhs.checked_add(rhs).ok_or(())
    }

    async fn on_notification(&self, method: &[u8], (text,): Self::Notification, _nvim: Neovim<W>) {
        assert_eq!(method, b"event");
        self.0.send(text.into_inner()).unwrap();
    }
}

#[tokio::test]
#[ignore = "requires a local Neovim executable; set NVIM_BIN"]
async fn plugin_handles_nested_requests_and_owned_notifications() {
    let bin = std::env::var_os("NVIM_BIN").unwrap_or_else(|| "nvim".into());
    let mut child = Command::new(bin)
        .args(["--embed", "--headless", "--clean", "-n", "-i", "NONE"])
        .env("NVIM_LOG_FILE", "/tmp/nvim-api-plugin-integration.log")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("could not start Neovim; set NVIM_BIN to its executable");
    let (events, mut received) = mpsc::unbounded_channel();
    let (nvim, task) = spawn(
        child.stdout.take().unwrap(),
        child.stdin.take().unwrap(),
        Plugin(events),
    );

    timeout(Duration::from_secs(10), async {
        let mut info: Array = nvim.request::<api::NvimGetApiInfo, _>(()).await.unwrap();
        let channel =
            <i64 as Decode<sys::types::Object, Bytes>>::decode(info.next().unwrap()).unwrap();
        nvim.request::<api::NvimSetCurrentLine, ()>(("before",))
            .await
            .unwrap();
        let result: i32 = nvim
            .request::<api::NvimExecLua, _>((
                "local chan = ...; vim.rpcnotify(chan, 'event', 'owned'); \
                 return vim.rpcrequest(chan, 'add', 20, 22, 'argument')",
                (channel,),
            ))
            .await
            .unwrap();
        assert_eq!(result, 42);
        let event = received.recv().await.unwrap();

        nvim.notify::<api::NvimSetCurrentLine>(("after",))
            .await
            .unwrap();
        let line: Bin = nvim
            .request::<api::NvimGetCurrentLine, _>(())
            .await
            .unwrap();
        assert_eq!(line.as_slice(), b"after");
        assert_eq!(&event[..], b"owned");
    })
    .await
    .expect("plugin RPC timed out");

    task.abort();
    let _ = task.await;
    child.kill().await.unwrap();
    child.wait().await.unwrap();
}
