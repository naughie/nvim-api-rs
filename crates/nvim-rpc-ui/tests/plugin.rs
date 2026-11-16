use nvim_rpc::Receiver;
use nvim_rpc::mp::encode::WriterSyncExt as _;
use nvim_rpc::mp::encode::primitive::Nil;
use nvim_rpc::read::TokioRead;
use nvim_rpc::rpc::Message;

use nvim_rpc_plugin::{Array, ConnectionError, DecodeError, Handler, Neovim, api, spawn};

use nvim_rpc_ui::{Backend, Event, Frame, Limits, ProtocolError};
use nvim_rpc_ui::{FrameError as Error, UiHandler};

use tokio::io::{AsyncWrite, AsyncWriteExt as _, duplex, split};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

struct Application {
    events: mpsc::UnboundedSender<String>,
}

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Application {
    type Request = ();
    type Response = Result<i32, Infallible>;
    type Notification = (String,);

    async fn on_request(&self, method: &[u8], _: (), _: Neovim<W>) -> Self::Response {
        assert_eq!(method, b"custom_request");
        self.events.send("request".into()).unwrap();
        Ok(77)
    }

    async fn on_notification(&self, method: &[u8], (text,): (String,), nvim: Neovim<W>) {
        assert_eq!(method, b"nested");
        assert_eq!(text, "argument");
        let reply = nvim
            .request::<api::NvimGetCurrentLine, String>(())
            .await
            .unwrap();
        self.events.send(reply).unwrap();
    }
}

struct LocalBackend(Rc<RefCell<Vec<Frame>>>);

impl Backend for LocalBackend {
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.0.borrow_mut().push(frame);
        Ok(())
    }
}

#[tokio::test]
async fn slow_presentation_preserves_events_and_custom_callbacks_can_make_nested_calls() {
    timeout(Duration::from_secs(2), async {
        let (transport, peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (peer_reader, mut peer_writer) = split(peer);
        let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
        let (events, mut received) = mpsc::unbounded_channel();
        let (handler, mut frames) = UiHandler::new(Application { events }, Limits::default());
        let (nvim, driver) = spawn(reader, writer, handler);
        let mut bytes = Vec::new();
        bytes
            .write_primitive((
                2u8,
                "redraw",
                (("set_title", ("one",)), ("bell", ()), ("flush", ())),
            ))
            .consume();
        bytes
            .write_primitive((
                2u8,
                "redraw",
                (("set_title", ("two",)), ("visual_bell", ()), ("flush", ())),
            ))
            .consume();
        bytes
            .write_primitive((
                2u8,
                "redraw",
                (("set_title", ("three",)), ("bell", ()), ("flush", ())),
            ))
            .consume();
        bytes
            .write_primitive((
                2u8,
                "redraw",
                (("set_title", ("four",)), ("visual_bell", ()), ("flush", ())),
            ))
            .consume();
        bytes
            .write_primitive((2u8, "nested", ("argument",)))
            .consume();
        bytes
            .write_primitive((0u8, 91u32, "custom_request", ()))
            .consume();
        peer_writer.write_all(&bytes).await.unwrap();

        // No frame has been consumed: reaching this call proves a full frame
        // queue did not prevent later custom notifications from running.
        let Message::Request(request) = receiver.recv_async().await.unwrap() else {
            panic!("expected the nested API call");
        };
        assert_eq!(request.method.as_slice(), b"nvim_get_current_line");
        bytes.clear();
        bytes
            .write_primitive((1u8, request.msgid.0, Nil, "nested result"))
            .consume();
        peer_writer.write_all(&bytes).await.unwrap();
        assert_eq!(received.recv().await.as_deref(), Some("nested result"));
        assert_eq!(received.recv().await.as_deref(), Some("request"));
        let Message::Response(response) = receiver.recv_async().await.unwrap() else {
            panic!("expected the custom request's reply");
        };
        assert_eq!(response.msgid.0, 91);
        assert!(matches!(
            response.result,
            Ok(nvim_rpc::mp::decode::Object::U8(77))
        ));

        let painted = Rc::new(RefCell::new(Vec::new()));
        let mut backend = LocalBackend(Rc::clone(&painted));
        assert!(frames.present_next(&mut backend).await.unwrap());
        {
            let painted = painted.borrow();
            let latest = &painted[0];
            assert_eq!(latest.sequence, 4);
            assert_eq!(latest.state.title.as_ref(), "four");
            assert!(latest.damage.full);
            assert!(matches!(
                latest.events.as_slice(),
                [
                    Event::SetTitle(one),
                    Event::Bell,
                    Event::SetTitle(two),
                    Event::VisualBell,
                    Event::SetTitle(three),
                    Event::Bell,
                    Event::SetTitle(four),
                    Event::VisualBell
                ] if one.as_ref() == "one" && two.as_ref() == "two"
                    && three.as_ref() == "three" && four.as_ref() == "four"
            ));
        }
        nvim.close();
        assert!(matches!(
            driver.await.unwrap(),
            Err(ConnectionError::Closed)
        ));
        assert!(!frames.present_next(&mut backend).await.unwrap());
    })
    .await
    .expect("shared UI and plugin dispatch timed out");
}

struct Ignore;

impl<W: AsyncWrite + Unpin + Send + 'static> Handler<W> for Ignore {
    type Request = Array;
    type Response = ();
    type Notification = (String,);

    async fn on_request(&self, _: &[u8], _: Array, _: Neovim<W>) {}
    async fn on_notification(&self, _: &[u8], _: (String,), _: Neovim<W>) {
        panic!("invalid arguments must not reach the handler");
    }
}

#[tokio::test]
async fn coalesced_event_limit_delivers_valid_prefix_then_closes_the_connection() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (handler, mut frames) = UiHandler::new(
            Ignore,
            Limits {
                max_pending_events: 1,
                ..Limits::default()
            },
        );
        let (nvim, driver) = spawn(reader, writer, handler);
        let mut bytes = Vec::new();
        for _ in 0..4 {
            bytes
                .write_primitive((2u8, "redraw", (("bell", ()), ("flush", ()))))
                .consume();
        }
        peer.write_all(&bytes).await.unwrap();
        // Wait for closure without draining, ensuring the pending limit is hit.
        assert!(matches!(
            driver.await.unwrap(),
            Err(ConnectionError::Closed)
        ));
        let frame = frames.recv().await.unwrap().unwrap();
        assert_eq!(frame.sequence, 1);
        assert!(matches!(frame.events.as_slice(), [Event::Bell]));
        assert!(matches!(
            frames.recv().await,
            Some(Err(Error::Protocol(ProtocolError::Limit("pending events"))))
        ));
        assert!(frames.recv().await.is_none());
        assert!(
            nvim.request::<api::NvimGetCurrentLine, String>(())
                .await
                .is_err()
        );
    })
    .await
    .expect("pending-event overflow did not close the connection");
}

#[tokio::test]
async fn delegated_notification_decode_failure_is_preserved_after_valid_frames() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (handler, mut frames) = UiHandler::new(Ignore, Limits::default());
        let (_, driver) = spawn(reader, writer, handler);
        let mut bytes = Vec::new();
        bytes
            .write_primitive((2u8, "redraw", (("flush", ()),)))
            .consume();
        bytes.write_primitive((2u8, "application", ())).consume();
        bytes
            .write_primitive((2u8, "redraw", (("flush", ()),)))
            .consume();
        peer.write_all(&bytes).await.unwrap();
        assert!(matches!(
            driver.await.unwrap(),
            Err(ConnectionError::Closed)
        ));
        assert_eq!(frames.recv().await.unwrap().unwrap().sequence, 1);
        assert!(matches!(
            frames.recv().await,
            Some(Err(Error::Notification(DecodeError::InvalidLength {
                expected: 1,
                actual: 0
            })))
        ));
        assert!(frames.recv().await.is_none());
    })
    .await
    .expect("invalid application notification did not close the connection");
}

#[tokio::test]
async fn malformed_ui_notification_wakes_the_frame_receiver_with_its_error() {
    timeout(Duration::from_secs(2), async {
        let (transport, mut peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (handler, mut frames) = UiHandler::new(Ignore, Limits::default());
        let (_, driver) = spawn(reader, writer, handler);
        let waiting = frames.recv();
        let mut bytes = Vec::new();
        bytes.write_primitive((2u8, "redraw", (("grid_destroy", (99,)), ("flush", ())))).consume();
        let send = async { peer.write_all(&bytes).await.unwrap(); };
        let (frame, ()) = tokio::join!(waiting, send);
        assert!(matches!(frame, Some(Err(Error::Protocol(ProtocolError::Invalid { ref event, .. }))) if event == "grid_destroy"));
        assert!(frames.recv().await.is_none());
        assert!(matches!(driver.await.unwrap(), Err(ConnectionError::Closed)));
    }).await.expect("UI protocol failure did not wake frame reception");
}

#[tokio::test]
async fn local_backend_failure_preserves_its_error_and_leaves_rpc_available() {
    struct FailingBackend {
        error: Rc<str>,
        frames: Vec<Frame>,
    }

    impl Backend for FailingBackend {
        type Error = Rc<str>;

        fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
            self.frames.push(frame);
            Err(Rc::clone(&self.error))
        }
    }

    timeout(Duration::from_secs(2), async {
        let (transport, peer) = duplex(4096);
        let (reader, writer) = split(transport);
        let (peer_reader, mut peer_writer) = split(peer);
        let mut receiver = Receiver::new(TokioRead(peer_reader), [0; 4096]);
        let (handler, mut frames) = UiHandler::new(Ignore, Limits::default());
        let (nvim, driver) = spawn(reader, writer, handler);
        let error = Rc::<str>::from("toolkit surface unavailable");
        let mut backend = FailingBackend {
            error: Rc::clone(&error),
            frames: Vec::new(),
        };
        let mut bytes = Vec::new();
        bytes
            .write_primitive((2u8, "redraw", (("bell", ()), ("flush", ()))))
            .consume();
        peer_writer.write_all(&bytes).await.unwrap();
        let Err(nvim_rpc_ui::PresentationError::Backend(actual)) =
            frames.present_next(&mut backend).await
        else {
            panic!("expected the concrete backend error");
        };
        assert!(Rc::ptr_eq(&actual, &error));
        assert!(matches!(backend.frames[0].events.as_slice(), [Event::Bell]));
        nvim.notify::<api::NvimSetCurrentLine>(("still connected",))
            .await
            .unwrap();
        let Message::Notification(notification) = receiver.recv_async().await.unwrap() else {
            panic!("expected the API notification after presentation failed");
        };
        assert_eq!(notification.method.as_slice(), b"nvim_set_current_line");
        bytes.clear();
        bytes
            .write_primitive((2u8, "redraw", (("flush", ()),)))
            .consume();
        peer_writer.write_all(&bytes).await.unwrap();
        assert_eq!(frames.recv().await.unwrap().unwrap().sequence, 2);
        nvim.close();
        assert!(matches!(
            driver.await.unwrap(),
            Err(ConnectionError::Closed)
        ));
        assert!(frames.recv().await.is_none());
    })
    .await
    .expect("presentation failure made the RPC connection unavailable");
}
