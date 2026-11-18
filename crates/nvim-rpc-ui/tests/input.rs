use nvim_rpc::api_types::Decode;
use nvim_rpc::mp::encode::TokioWrite;
use nvim_rpc::read::TokioRead;
use nvim_rpc::sys::types;
use nvim_rpc::{Neovim, Receiver, rpc::Message};

use nvim_rpc_plugin::{Array, ConnectionError, Handler, Neovim as PluginNeovim};
use nvim_rpc_ui::{Client, ClientError};

use tokio::io::{DuplexStream, WriteHalf, duplex, split};
use tokio::task::JoinHandle;
use tokio::time::{Duration, timeout};

type Writer = WriteHalf<DuplexStream>;
type RpcTask = JoinHandle<Result<(), ConnectionError>>;

struct Ignore;

impl Handler<Writer> for Ignore {
    type Request = Array;
    type Response = ();
    type Notification = Array;

    async fn on_request(&self, _: &[u8], _: Array, _: PluginNeovim<Writer>) {}
    async fn on_notification(&self, _: &[u8], _: Array, _: PluginNeovim<Writer>) {}
}

fn connection() -> (Client<Writer>, RpcTask, DuplexStream) {
    let (local, remote) = duplex(65536);
    let (r, w) = split(local);
    let (nvim, task) = nvim_rpc_plugin::spawn(r, w, Ignore);
    (Client::new(nvim), task, remote)
}

async fn close(client: Client<Writer>, task: RpcTask, peer: JoinHandle<()>) {
    client.nvim().close();
    let result = timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(result, Ok(()) | Err(ConnectionError::Closed)));
    peer.abort();
    if let Err(e) = peer.await {
        assert!(e.is_cancelled(), "peer failed: {e}");
    }
}

#[tokio::test]
async fn input_retries_zero_and_partial_utf8_byte_acceptance_in_order() {
    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        // A byte boundary may be inside UTF-8; the writer must preserve bytes
        // rather than slicing a str at the server's reported offset.
        for (expected, accepted) in [
            (&b"a\xe6\x97\xa5b"[..], 0),
            (&b"a\xe6\x97\xa5b"[..], 2),
            (&b"\x97\xa5b"[..], 3),
        ] {
            let Message::Request(mut request) = receiver.recv_async().await.unwrap() else {
                panic!()
            };
            assert_eq!(request.method.as_slice(), b"nvim_input");
            let bytes = <&[u8] as Decode<types::String, _>>::decode(request.params.next().unwrap())
                .unwrap();
            assert_eq!(bytes, expected);
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(accepted),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        }
        receiver.recv_async().await.ok();
    });
    timeout(Duration::from_secs(2), client.input("a日b"))
        .await
        .unwrap()
        .unwrap();
    close(client, task, peer).await;
}

#[tokio::test]
async fn concurrent_input_calls_do_not_interleave_partial_sequences() {
    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let (seen, start_second) = tokio::sync::oneshot::channel();
    let (second_started, continue_first) = tokio::sync::oneshot::channel();
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        let mut seen = Some(seen);
        let mut continue_first = Some(continue_first);
        for (expected, accepted) in [("abc", 1), ("bc", 2), ("xyz", 3)] {
            let Message::Request(mut request) = receiver.recv_async().await.unwrap() else {
                panic!()
            };
            let text =
                <&str as Decode<types::String, _>>::decode(request.params.next().unwrap()).unwrap();
            assert_eq!(text, expected);
            if expected == "abc" {
                seen.take().unwrap().send(()).unwrap();
                continue_first.take().unwrap().await.unwrap();
            }
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(accepted),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        }
        receiver.recv_async().await.ok();
    });
    let first_client = client.clone();
    let first = tokio::spawn(async move { first_client.input("abc").await });
    start_second.await.unwrap();
    let second_client = client.clone();
    let second = tokio::spawn(async move {
        second_started.send(()).unwrap();
        second_client.input("xyz").await
    });
    timeout(Duration::from_secs(2), async {
        first.await.unwrap().unwrap();
        second.await.unwrap().unwrap();
    })
    .await
    .unwrap();
    close(client, task, peer).await;
}

#[tokio::test]
async fn cancelling_zero_acceptance_input_releases_sequence_lock() {
    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let (seen_tx, seen_rx) = tokio::sync::oneshot::channel();
    let (resume_tx, resume_rx) = tokio::sync::oneshot::channel();
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        let Message::Request(request) = receiver.recv_async().await.unwrap() else {
            panic!()
        };
        sender
            .send_async(
                nvim_rpc::mp::encode::rpc::Response {
                    msgid: request.msgid,
                    result: Ok::<_, &str>(0),
                }
                .to_primitive(),
            )
            .await
            .unwrap();
        sender.flush_async().await.unwrap();
        seen_tx.send(()).unwrap();
        resume_rx.await.unwrap();
        loop {
            let Message::Request(mut request) = receiver.recv_async().await.unwrap() else {
                panic!()
            };
            let text = <&[u8] as Decode<types::String, _>>::decode(request.params.next().unwrap())
                .unwrap();
            let accepted = text.len();
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(accepted),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
            if text == b"b" {
                break;
            }
        }
        receiver.recv_async().await.ok();
    });
    let first_client = client.clone();
    let first = tokio::spawn(async move { first_client.input("a").await });
    seen_rx.await.unwrap();
    // Let the response dispatcher release the request's write guard before abort.
    tokio::task::yield_now().await;
    first.abort();
    let _ = first.await;
    resume_tx.send(()).unwrap();
    timeout(Duration::from_secs(2), client.input("b"))
        .await
        .unwrap()
        .unwrap();
    close(client, task, peer).await;
}

#[tokio::test]
async fn impossible_input_acceptance_closes_the_connection() {
    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        let Message::Request(request) = receiver.recv_async().await.unwrap() else {
            panic!()
        };
        sender
            .send_async(
                nvim_rpc::mp::encode::rpc::Response {
                    msgid: request.msgid,
                    result: Ok::<_, &str>(2),
                }
                .to_primitive(),
            )
            .await
            .unwrap();
        sender.flush_async().await.unwrap();
        receiver.recv_async().await.ok();
    });
    let error = timeout(Duration::from_secs(2), client.input("x"))
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        error,
        ClientError::InvalidInputAcceptance {
            accepted: 2,
            remaining: 1
        }
    ));
    let error = timeout(Duration::from_secs(2), client.input("y"))
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        error,
        ClientError::Rpc(nvim_rpc_plugin::Error::Connection(ConnectionError::Closed))
    ));
    close(client, task, peer).await;
}

#[tokio::test]
async fn attach_detach_and_ordinary_requests_use_the_plugin_client() {
    use nvim_rpc::api;
    use nvim_rpc::mp::encode::primitive::Nil;
    use nvim_rpc_ui::UiOptions;

    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        for (id, method) in [
            b"nvim_ui_attach".as_slice(),
            b"nvim_get_current_line",
            b"nvim_ui_detach",
        ]
        .into_iter()
        .enumerate()
        {
            let Message::Request(mut request) = receiver.recv_async().await.unwrap() else {
                panic!("expected RPC request");
            };
            assert_eq!(request.msgid.0, id as u32);
            assert_eq!(request.method.as_slice(), method);
            if method == b"nvim_ui_attach" {
                let width =
                    <i64 as Decode<types::Int, _>>::decode(request.params.next().unwrap()).unwrap();
                let height =
                    <i64 as Decode<types::Int, _>>::decode(request.params.next().unwrap()).unwrap();
                assert_eq!((width, height), (80, 24));
                let options = <nvim_rpc::mp::decode::Map<_> as Decode<types::Dict, _>>::decode(
                    request.params.next().unwrap(),
                )
                .unwrap();
                let linegrid = options.into_iter().find_map(|entry| {
                    let key = <&str as Decode<types::String, _>>::decode(entry.key).unwrap();
                    (key == "ext_linegrid")
                        .then(|| <bool as Decode<types::Bool, _>>::decode(entry.value).unwrap())
                });
                assert_eq!(linegrid, Some(true));
            } else {
                assert_eq!(request.params.len(), 0);
            }
            if method == b"nvim_get_current_line" {
                sender
                    .send_async(
                        nvim_rpc::mp::encode::rpc::Response {
                            msgid: request.msgid,
                            result: Ok::<_, &str>("hello"),
                        }
                        .to_primitive(),
                    )
                    .await
                    .unwrap();
            } else {
                sender
                    .send_async(
                        nvim_rpc::mp::encode::rpc::Response {
                            msgid: request.msgid,
                            result: Ok::<_, &str>(Nil),
                        }
                        .to_primitive(),
                    )
                    .await
                    .unwrap();
            }
            sender.flush_async().await.unwrap();
        }
        receiver.recv_async().await.ok();
    });

    timeout(Duration::from_secs(2), async {
        client
            .attach(
                80,
                24,
                UiOptions {
                    ext_linegrid: true,
                    ..UiOptions::default()
                },
            )
            .await
            .unwrap();
        let line = client
            .nvim()
            .request::<api::NvimGetCurrentLine, String>(())
            .await
            .unwrap();
        assert_eq!(line, "hello");
        client.detach().await.unwrap();
    })
    .await
    .unwrap();
    close(client, task, peer).await;
}

#[tokio::test]
async fn invalid_ui_options_and_sizes_leave_the_rpc_client_usable() {
    use nvim_rpc_ui::UiOptions;

    let (client, task, remote) = connection();
    assert!(matches!(
        client.attach(0, 24, UiOptions::default()).await,
        Err(ClientError::InvalidSize {
            width: 0,
            height: 24
        })
    ));
    assert!(matches!(
        client.attach(80, 24, UiOptions::default()).await,
        Err(ClientError::InvalidOptions)
    ));
    assert!(matches!(
        client.resize(80, 0).await,
        Err(ClientError::InvalidSize {
            width: 80,
            height: 0
        })
    ));

    let (seen_tx, seen_rx) = tokio::sync::oneshot::channel();
    let (r, _w) = split(remote);
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let Message::Notification(mut notification) = receiver.recv_async().await.unwrap() else {
            panic!("expected focus notification");
        };
        assert_eq!(notification.method.as_slice(), b"nvim_ui_set_focus");
        assert!(
            <bool as Decode<types::Bool, _>>::decode(notification.params.next().unwrap()).unwrap()
        );
        seen_tx.send(()).unwrap();
        receiver.recv_async().await.ok();
    });
    timeout(Duration::from_secs(2), client.focus(true))
        .await
        .unwrap()
        .unwrap();
    timeout(Duration::from_secs(2), seen_rx)
        .await
        .unwrap()
        .unwrap();
    close(client, task, peer).await;
}

#[tokio::test]
async fn portable_keys_and_long_sequences_share_partial_acceptance_retry() {
    use nvim_rpc_ui::{KeyInput, Modifiers, SpecialKey};
    let (client, task, remote) = connection();
    let (r, w) = split(remote);
    let long = "a".repeat(500);
    let expected = [
        "<S-C-M-D-PageDown>".to_owned(),
        "<M-ä>".to_owned(),
        "日<LT>Esc>".to_owned(),
        long.clone(),
        long[137..].to_owned(),
    ];
    let peer = tokio::spawn(async move {
        let mut receiver = Receiver::new(TokioRead(r), vec![0; 65536]);
        let mut sender = Neovim::new(TokioWrite::new(w));
        for (i, expected) in expected.iter().enumerate() {
            let Message::Request(mut request) = receiver.recv_async().await.unwrap() else {
                panic!("expected nvim_input request")
            };
            assert_eq!(request.method.as_slice(), b"nvim_input");
            let keys =
                <&str as Decode<types::String, _>>::decode(request.params.next().unwrap()).unwrap();
            assert_eq!(keys, expected);
            let accepted = if i == 3 { 137 } else { keys.len() };
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(accepted as i64),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        }
        receiver.recv_async().await.ok();
    });
    timeout(Duration::from_secs(2), async {
        client
            .send_key(KeyInput::SpecialKey {
                key: SpecialKey::PageDown,
                modifiers: Modifiers {
                    shift: true,
                    control: true,
                    alt: true,
                    command: true,
                },
            })
            .await
            .unwrap();
        client
            .send_key(KeyInput::from_char(
                'ä',
                Modifiers {
                    alt: true,
                    ..Default::default()
                },
            ))
            .await
            .unwrap();
        client
            .send_key(KeyInput::Text { text: "日<Esc>" })
            .await
            .unwrap();
        client
            .send_key(KeyInput::Sequence { keys: &long })
            .await
            .unwrap();
    })
    .await
    .unwrap();
    close(client, task, peer).await;
}
