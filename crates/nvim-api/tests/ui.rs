use nvim_api::mp::decode::Object;
use nvim_api::mp::encode::WriterSyncExt as _;
use nvim_api::rpc::Message;
use nvim_api::{ui, ui_events};

#[test]
fn redraw_batches_preserve_repeated_calls_unknown_events_and_extra_arguments() {
    let batch = nvim_api::mp::encode::rpc::Notification {
        method: "redraw",
        params: (
            ("grid_resize", (1, 80, 24, "future argument"), (2, 40, 10)),
            ("future_event", ("opaque",)),
            ("flush", ()),
        ),
    }
    .to_primitive();
    let mut bytes = Vec::new();
    bytes.write_primitive(batch).consume();
    let Message::Notification(notification) = nvim_api::rpc::decode(&mut &bytes[..]).unwrap()
    else {
        panic!("expected redraw notification");
    };
    let mut redraw = ui::Redraw::new(notification).unwrap();
    assert_eq!(redraw.len(), 3);

    let mut resize = redraw.next().unwrap().unwrap();
    assert!(resize.is::<ui_events::GridResize>());
    assert!(resize.decode_next::<ui_events::Flush, ()>().is_err());
    assert_eq!(
        resize
            .decode_next::<ui_events::GridResize, (i64, i64, i64)>()
            .unwrap(),
        Some((1, 80, 24))
    );
    assert_eq!(
        resize
            .decode_next::<ui_events::GridResize, (i64, i64, i64)>()
            .unwrap(),
        Some((2, 40, 10))
    );
    assert_eq!(
        resize
            .decode_next::<ui_events::GridResize, (i64, i64, i64)>()
            .unwrap(),
        None
    );

    let mut unknown = redraw.next().unwrap().unwrap();
    assert_eq!(unknown.name().as_slice(), b"future_event");
    let mut args = unknown.next().unwrap().unwrap();
    let Some(Object::Str(text)) = args.next() else {
        panic!("expected unknown event argument");
    };
    assert_eq!(text.as_slice(), b"opaque");

    let mut flush = redraw.next().unwrap().unwrap();
    assert_eq!(
        flush.decode_next::<ui_events::Flush, ()>().unwrap(),
        Some(())
    );
    assert!(redraw.next().is_none());
}
