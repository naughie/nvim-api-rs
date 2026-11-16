use nvim_rpc::mp::encode::{Primitive, TokioWrite, WriterSyncExt as _};
use nvim_rpc::read::TokioRead;
use nvim_rpc::{Neovim, Receiver, api, rpc::Message};

use nvim_rpc_ui::{Backend, Event, Frame};
use nvim_rpc_ui::{ConnectionOptions, PresentationError, connect as connect_with_handler};

use tokio::io::{AsyncWriteExt as _, duplex, split};
use tokio::time::{Duration, timeout};

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

struct RecordingBackend(Rc<RefCell<Vec<Frame>>>);

impl Backend for RecordingBackend {
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.0.borrow_mut().push(frame);
        Ok(())
    }
}

fn redraw(params: impl Primitive) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    bytes
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

#[tokio::test]
async fn local_backend_presents_while_rpc_responses_progress() {
    timeout(Duration::from_secs(2), async {
        let (local, remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        let (r, w) = split(remote);
        let peer = tokio::spawn(async move {
            let mut sender = Neovim::new(TokioWrite::new(w));
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Notification {
                        method: "redraw",
                        params: (("bell", ()), ("flush", ())),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
            let mut receiver = Receiver::new(TokioRead(r), vec![0; 4096]);
            let Message::Request(request) = receiver.recv_async().await.unwrap() else {
                panic!("expected an API request");
            };
            assert_eq!(request.method.as_slice(), b"nvim_eval");
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(42i64),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        });

        let frames = Rc::new(RefCell::new(Vec::new()));
        let mut backend = RecordingBackend(Rc::clone(&frames));
        let client = connection.client.clone();
        let present = async {
            assert!(connection.present_next(&mut backend).await.unwrap());
            assert!(!connection.present_next(&mut backend).await.unwrap());
        };
        let (_, result) = tokio::join!(
            present,
            client.nvim().request::<api::NvimEval, i64>(("6 * 7",))
        );
        assert_eq!(result.unwrap(), 42);
        peer.await.unwrap();
        let frames = frames.borrow();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].sequence, 1);
        assert!(matches!(frames[0].events.as_slice(), [Event::Bell]));
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn queued_frames_coalesce_ordered_events_before_repeated_eof() {
    timeout(Duration::from_secs(2), async {
        let (local, mut remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        remote
            .write_all(&redraw((
                ("set_title", ("first",)),
                ("flush", ()),
                ("set_title", ("second",)),
                ("flush", ()),
                ("set_title", ("third",)),
                ("flush", ()),
            )))
            .await
            .unwrap();
        let (r, w) = split(remote);
        let peer = tokio::spawn(async move {
            let mut receiver = Receiver::new(TokioRead(r), vec![0; 4096]);
            let Message::Request(request) = receiver.recv_async().await.unwrap() else {
                panic!("expected a request to confirm receive progress");
            };
            let mut sender = Neovim::new(TokioWrite::new(w));
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(42i64),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        });
        // The response follows all redraws on the wire, so they are queued now.
        assert_eq!(
            connection
                .client
                .nvim()
                .request::<api::NvimEval, i64>(("42",))
                .await
                .unwrap(),
            42
        );
        peer.await.unwrap();
        // Observe disconnect while completed frames remain queued.
        assert!(
            connection
                .client
                .nvim()
                .request::<api::NvimGetCurrentLine, String>(())
                .await
                .is_err()
        );
        let frames = Rc::new(RefCell::new(Vec::new()));
        let mut backend = RecordingBackend(Rc::clone(&frames));
        assert!(connection.present_next(&mut backend).await.unwrap());
        for _ in 0..2 {
            assert!(!connection.present_next(&mut backend).await.unwrap());
        }
        let frames = frames.borrow();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].sequence, 3);
        assert_eq!(frames[0].state.title.as_ref(), "third");
        assert!(frames[0].damage.full);
        let titles: Vec<_> = frames[0]
            .events
            .iter()
            .map(|event| {
                let Event::SetTitle(title) = event else {
                    panic!("expected title event");
                };
                title.as_ref()
            })
            .collect();
        assert_eq!(titles, ["first", "second", "third"]);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn cancelling_an_empty_receive_keeps_the_next_frame() {
    timeout(Duration::from_secs(2), async {
        let (local, mut remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        let frames = Rc::new(RefCell::new(Vec::new()));
        let mut backend = RecordingBackend(Rc::clone(&frames));
        let backend = &mut backend;
        tokio::select! {
            biased;
            result = connection.present_next(backend) => panic!("unexpected receive: {result:?}"),
            () = tokio::task::yield_now() => {},
        }
        assert!(frames.borrow().is_empty());
        remote
            .write_all(&redraw((("visual_bell", ()), ("flush", ()))))
            .await
            .unwrap();
        assert!(connection.present_next(backend).await.unwrap());
        let frames = frames.borrow();
        assert_eq!(frames.len(), 1);
        assert!(matches!(frames[0].events.as_slice(), [Event::VisualBell]));
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn backend_error_is_preserved_without_retrying_or_closing_rpc() {
    #[derive(Debug, PartialEq, Eq)]
    struct DrawError;

    struct FailingBackend(Vec<u64>);
    impl Backend for FailingBackend {
        type Error = DrawError;

        fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
            self.0.push(frame.sequence);
            if self.0.len() == 1 {
                Err(DrawError)
            } else {
                Ok(())
            }
        }
    }

    timeout(Duration::from_secs(2), async {
        let (local, remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        let (r, w) = split(remote);
        let peer = tokio::spawn(async move {
            let mut sender = Neovim::new(TokioWrite::new(w));
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Notification {
                        method: "redraw",
                        params: (("flush", ()),),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
            let mut receiver = Receiver::new(TokioRead(r), vec![0; 4096]);
            let Message::Request(request) = receiver.recv_async().await.unwrap() else {
                panic!("expected an API request after the backend error");
            };
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Response {
                        msgid: request.msgid,
                        result: Ok::<_, &str>(42i64),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender
                .send_async(
                    nvim_rpc::mp::encode::rpc::Notification {
                        method: "redraw",
                        params: (("flush", ()),),
                    }
                    .to_primitive(),
                )
                .await
                .unwrap();
            sender.flush_async().await.unwrap();
        });

        let mut backend = FailingBackend(Vec::new());
        assert!(matches!(
            connection.present_next(&mut backend).await,
            Err(PresentationError::Backend(DrawError))
        ));
        assert_eq!(
            connection
                .client
                .nvim()
                .request::<api::NvimEval, i64>(("42",))
                .await
                .unwrap(),
            42
        );
        assert!(connection.present_next(&mut backend).await.unwrap());
        assert_eq!(backend.0, [1, 2]);
        peer.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn session_errors_do_not_reach_the_backend() {
    timeout(Duration::from_secs(2), async {
        let (local, mut remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        let frames = Rc::new(RefCell::new(Vec::new()));
        let mut backend = RecordingBackend(Rc::clone(&frames));
        remote.write_all(b"\xc0").await.unwrap();
        assert!(matches!(
            connection.present_next(&mut backend).await,
            Err(PresentationError::Session(nvim_rpc_ui::Error::Connection(
                ConnectionError::InvalidFormat
            )))
        ));
        assert!(frames.borrow().is_empty());
        assert!(!connection.present_next(&mut backend).await.unwrap());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn valid_frames_precede_ui_protocol_failure_and_repeated_exhaustion() {
    timeout(Duration::from_secs(2), async {
        let (local, mut remote) = duplex(4096);
        let (r, w) = split(local);
        let mut connection = connect(r, w, ConnectionOptions::default()).unwrap();
        remote
            .write_all(&redraw((
                ("bell", ()),
                ("flush", ()),
                ("grid_resize", (1,)),
            )))
            .await
            .unwrap();

        let frames = Rc::new(RefCell::new(Vec::new()));
        let mut backend = RecordingBackend(Rc::clone(&frames));
        assert!(connection.present_next(&mut backend).await.unwrap());
        assert!(matches!(
            connection.present_next(&mut backend).await,
            Err(PresentationError::Session(nvim_rpc_ui::Error::Ui(
                nvim_rpc_ui::FrameError::Protocol(nvim_rpc_ui::ProtocolError::Invalid { .. })
            )))
        ));
        for _ in 0..2 {
            assert!(!connection.present_next(&mut backend).await.unwrap());
        }
        assert!(matches!(
            connection
                .client
                .nvim()
                .request::<api::NvimGetCurrentLine, String>(())
                .await,
            Err(nvim_rpc_plugin::Error::Connection(ConnectionError::Closed))
        ));
        let frames = frames.borrow();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].sequence, 1);
        assert!(frames[0].state.grids.is_empty());
        assert!(matches!(frames[0].events.as_slice(), [Event::Bell]));
    })
    .await
    .unwrap();
}
