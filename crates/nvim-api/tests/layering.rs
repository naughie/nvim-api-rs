use nvim_api::api_types::encode::Like;
use nvim_api::mp::encode::primitive::Func;
use nvim_api::mp::encode::{WriterAsync, WriterAsyncExt};
use nvim_api::rpc::Message;
use nvim_api::{Buffer, Neovim, Window, api, sys};

use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn handles_convert_to_primitives_at_the_neovim_boundary() {
    let buffer = Buffer(7);
    let window = Window(9);
    assert_eq!(<Buffer as Like<sys::ext_types::Buffer>>::encode(buffer), 7);
    assert_eq!(
        <&Buffer as Like<sys::ext_types::Buffer>>::encode(&buffer),
        7
    );
    assert_eq!(<Window as Like<sys::types::Object>>::encode(window), 9);

    let mut nvim = Neovim::new(Vec::new());
    nvim.request::<api::NvimSetCurrentBuf>((&buffer,)).unwrap();
    let bytes = nvim.into_inner();
    let Message::Request(mut request) = nvim_api::rpc::decode(&mut &bytes[..]).unwrap() else {
        panic!("expected request");
    };
    assert_eq!(request.method.as_slice(), b"nvim_set_current_buf");
    assert!(matches!(
        request.params.next(),
        Some(nvim_api::mp::decode::Object::U8(7))
    ));
}

#[test]
fn local_async_writers_need_no_send_bound_or_named_future() {
    struct Writer(Rc<RefCell<Vec<u8>>>);

    impl WriterAsync for Writer {
        type Result = ();

        async fn write_all<'a>(&'a mut self, buf: &'a [u8]) {
            self.0.borrow_mut().extend_from_slice(buf);
        }

        async fn flush(&mut self) {}
    }

    let bytes = Rc::new(RefCell::new(Vec::new()));
    let mut nvim = Neovim::new(Writer(Rc::clone(&bytes)));
    futures::executor::block_on(nvim.notify_async::<api::NvimSetCurrentLine>(("local",))).unwrap();
    futures::executor::block_on(nvim.flush_async());
    let bytes = bytes.borrow();
    let Message::Notification(notification) = nvim_api::rpc::decode(&mut &bytes[..]).unwrap()
    else {
        panic!("expected notification");
    };
    assert_eq!(notification.method.as_slice(), b"nvim_set_current_line");
}

#[test]
fn local_primitive_encoding_does_not_require_send() {
    let value = Rc::new(42u8);
    let primitive = Func(move || *value);
    let mut bytes = Vec::new();
    futures::executor::block_on(
        WriterAsyncExt::write_primitive(&mut bytes, primitive).consume_async(),
    );
    assert_eq!(bytes, [42]);
}
