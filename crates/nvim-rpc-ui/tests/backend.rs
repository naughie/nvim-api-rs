use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;

use nvim_rpc_ui::{Backend, Event, Frame, GridId, ProtocolError, RedrawError, Ui};

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

fn redraw<B: Backend>(ui: &mut Ui<B>, params: impl Primitive) -> Result<(), RedrawError<B::Error>> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap()
    else {
        panic!("expected redraw notification");
    };
    ui.redraw(notification)
}

struct RecordingBackend(Rc<RefCell<Vec<Frame>>>);

impl Backend for RecordingBackend {
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.0.borrow_mut().push(frame);
        Ok(())
    }
}

#[test]
fn notification_dispatch_borrows_arguments_and_presents_api_errors() {
    let frames = Rc::new(RefCell::new(Vec::new()));
    let mut ui = Ui::new(RecordingBackend(Rc::clone(&frames)));
    let mut bytes = Vec::new();
    bytes
        .write_primitive((2u8, "custom", ("argument",)))
        .consume();
    let Message::Notification(mut notification) =
        nvim_rpc::rpc::decode(&mut bytes.as_slice()).unwrap()
    else {
        panic!("expected notification");
    };
    assert!(!ui.handle_notification(&notification).unwrap());
    assert_eq!(notification.params.len(), 1);
    let Some(nvim_rpc::mp::decode::Object::Str(argument)) = notification.params.next() else {
        panic!("expected original argument");
    };
    assert_eq!(argument.as_slice(), b"argument");

    let mut bytes = Vec::new();
    bytes
        .write_primitive((2u8, "nvim_error_event", (0, "API error")))
        .consume();
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut bytes.as_slice()).unwrap()
    else {
        panic!("expected notification");
    };
    assert!(ui.handle_notification(&notification).unwrap());
    assert_eq!(notification.params.len(), 2);
    let frames = frames.borrow();
    assert_eq!(frames.len(), 1);
    assert!(
        matches!(frames[0].events.as_slice(), [Event::RpcError { kind: 0, message }] if &**message == "API error")
    );
    assert!(!ui.is_failed());
}

#[test]
fn borrowed_local_backend_receives_each_flush_across_notification_boundaries() {
    let frames = Rc::new(RefCell::new(Vec::new()));
    let mut backend = RecordingBackend(Rc::clone(&frames));
    let mut ui = Ui::new(&mut backend);
    redraw(
        &mut ui,
        (
            ("grid_resize", (1, 2, 1)),
            ("grid_line", (1, 0, 0, (("a", 0, 2),), false)),
        ),
    )
    .unwrap();
    assert!(frames.borrow().is_empty());

    redraw(
        &mut ui,
        (
            ("bell", ()),
            ("flush", ()),
            ("grid_line", (1, 0, 0, (("b", 1),), false)),
            ("visual_bell", ()),
            ("flush", ()),
        ),
    )
    .unwrap();

    let frames = frames.borrow();
    assert_eq!(frames.len(), 2);
    assert_eq!((frames[0].sequence, frames[1].sequence), (1, 2));
    assert!(matches!(frames[0].events.as_slice(), [Event::Bell]));
    assert!(matches!(frames[1].events.as_slice(), [Event::VisualBell]));
    let before = &frames[0].state.grids[&GridId(1)];
    let after = &frames[1].state.grids[&GridId(1)];
    assert_eq!(before.cell(0, 0).unwrap().text.as_ref(), "a");
    assert_eq!(after.cell(0, 0).unwrap().text.as_ref(), "b");
    assert_eq!(after.cell(0, 1).unwrap().text.as_ref(), "a");
    assert!(!ui.is_failed());
}

#[test]
fn backend_failure_preserves_its_error_and_stops_before_later_events() {
    #[derive(Debug)]
    struct PaintError {
        surface: Rc<str>,
        sequence: u64,
    }

    struct FailingBackend {
        surface: Rc<str>,
        frames: Vec<Frame>,
    }

    impl Backend for FailingBackend {
        type Error = PaintError;

        fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
            let sequence = frame.sequence;
            self.frames.push(frame);
            if sequence == 2 {
                Err(PaintError {
                    surface: Rc::clone(&self.surface),
                    sequence,
                })
            } else {
                Ok(())
            }
        }
    }

    let surface = Rc::<str>::from("editor surface");
    let mut ui = Ui::new(FailingBackend {
        surface: Rc::clone(&surface),
        frames: Vec::new(),
    });
    let error = redraw(
        &mut ui,
        (
            ("grid_resize", (1, 2, 1)),
            ("flush", ()),
            ("bell", ()),
            ("flush", ()),
            // This invalid event must not replace the backend's earlier error.
            ("grid_destroy", (99,)),
            ("flush", ()),
        ),
    )
    .unwrap_err();
    let RedrawError::Backend(error) = error else {
        panic!("expected the backend's concrete error");
    };
    assert!(Rc::ptr_eq(&error.surface, &surface));
    assert_eq!(error.sequence, 2);
    assert_eq!(ui.backend().frames.len(), 2);
    assert!(matches!(
        ui.backend().frames[1].events.as_slice(),
        [Event::Bell]
    ));
    assert!(ui.is_failed());
    assert!(matches!(
        redraw(&mut ui, (("flush", ()),)),
        Err(RedrawError::Protocol(ProtocolError::Failed))
    ));
    assert_eq!(ui.backend().frames.len(), 2);
}

#[test]
fn protocol_failure_presents_only_preceding_complete_frames() {
    let frames = Rc::new(RefCell::new(Vec::new()));
    let mut ui = Ui::new(RecordingBackend(Rc::clone(&frames)));
    let error = redraw(
        &mut ui,
        (
            ("grid_resize", (1, 2, 1)),
            ("grid_line", (1, 0, 0, (("a", 0, 2),), false)),
            ("flush", ()),
            ("grid_line", (1, 0, 1, (("b", 0, 2),), false)),
            ("flush", ()),
        ),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        RedrawError::Protocol(ProtocolError::Invalid { ref event, .. })
            if event == "grid_line"
    ));
    assert!(ui.is_failed());
    assert_eq!(frames.borrow().len(), 1);
    assert_eq!(
        frames.borrow()[0].state.grids[&GridId(1)]
            .cell(0, 1)
            .unwrap()
            .text
            .as_ref(),
        "a"
    );
    assert!(matches!(
        redraw(&mut ui, (("flush", ()),)),
        Err(RedrawError::Protocol(ProtocolError::Failed))
    ));
    assert_eq!(frames.borrow().len(), 1);
}

#[test]
fn backend_can_repaint_a_retained_frame_after_the_driver_is_dropped() {
    struct Painter {
        latest: Option<Frame>,
        painted: String,
    }

    impl Painter {
        fn repaint(&mut self) {
            self.painted.clear();
            if let Some(frame) = &self.latest {
                for cell in &frame.state.grids[&GridId(1)].rows[0].cells {
                    self.painted.push_str(&cell.text);
                }
            }
        }
    }

    impl Backend for Painter {
        type Error = Infallible;

        fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
            self.latest = Some(frame);
            Ok(())
        }
    }

    let mut ui = Ui::new(Painter {
        latest: None,
        painted: String::new(),
    });
    redraw(
        &mut ui,
        (
            ("grid_resize", (1, 2, 1)),
            ("grid_line", (1, 0, 0, (("界", 0), ("",)), false)),
            ("flush", ()),
        ),
    )
    .unwrap();
    ui.backend_mut().repaint();
    assert_eq!(ui.backend().painted, "界");
    let mut backend = ui.into_backend();
    backend.repaint();
    assert_eq!(backend.painted, "界");
    assert_eq!(backend.latest.as_ref().unwrap().sequence, 1);
}
