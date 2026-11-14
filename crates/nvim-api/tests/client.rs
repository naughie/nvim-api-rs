use nvim_api::api_types::decode::{Decode, DecodeError};
use nvim_api::api_types::encode::Like;
use nvim_api::mp::decode::{Buf, Object};
use nvim_api::mp::encode::{StdWrite, WriterAsync};
use nvim_api::rpc::{Message, MessageId, Response};
use nvim_api::{Neovim, Pending, ResponseError, SendError, api, sys};

use std::future::Future;
use std::io::{self, Cursor};
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

struct Line<'a>(&'a str);

impl<'a> Like<sys::types::String> for Line<'a> {
    type EncodeAs = &'a str;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

#[derive(Debug, PartialEq)]
struct Count(i64);

impl<B: Buf> Decode<sys::types::Object, B> for Count {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <i64 as Decode<sys::types::Object, B>>::decode(value).map(Self)
    }
}

#[test]
fn custom_arguments_and_decoders_keep_the_wire_contract() {
    let mut output = [0; 128];
    let mut cursor = Cursor::new(&mut output[..]);
    {
        let mut nvim = Neovim::new(StdWrite::new(&mut cursor));
        let pending = nvim
            .request::<api::NvimSetCurrentLine>((Line("borrowed"),))
            .unwrap();
        assert_eq!(pending.id(), MessageId(0));
    }
    let len = cursor.position() as usize;
    let mut bytes = &cursor.get_ref()[..len];
    let Message::Request(mut request) = nvim_api::rpc::decode(&mut bytes).unwrap() else {
        panic!("expected request");
    };
    assert_eq!(request.method.as_slice(), b"nvim_set_current_line");
    let Some(Object::Str(line)) = request.params.next() else {
        panic!("expected line argument");
    };
    assert_eq!(line.as_slice(), b"borrowed");
    assert!(request.params.next().is_none());

    let pending = Pending::<sys::functions::NvimEval>::new(MessageId(7));
    let value: Count = pending
        .decode(Response::<&[u8]> {
            msgid: MessageId(7),
            result: Ok(Object::U8(42)),
        })
        .unwrap();
    assert_eq!(value, Count(42));
}

#[test]
fn dictionary_arguments_encode_as_maps_including_empty_options() {
    use nvim_api::mp::encode::primitive::Map;

    let mut nvim = Neovim::new(Vec::new());
    nvim.request::<api::NvimGetOptionValue>(("number", Map::<[(&str, bool); 0]>([])))
        .unwrap();
    nvim.request::<api::NvimGetOptionValue>(("number", Map([("scope", "local")])))
        .unwrap();
    nvim.request::<api::NvimGetOptionValue>(("number", Map([(b"scope".as_slice(), "local")])))
        .unwrap();

    let bytes = nvim.into_inner();
    let mut bytes = bytes.as_slice();
    for len in [0, 1, 1] {
        let Message::Request(mut request) = nvim_api::rpc::decode(&mut bytes).unwrap() else {
            panic!("expected request");
        };
        assert_eq!(request.method.as_slice(), b"nvim_get_option_value");
        request.params.next().unwrap();
        let Some(Object::Map(opts)) = request.params.next() else {
            panic!("expected options dictionary");
        };
        assert_eq!(opts.remaining_len(), len);
        assert!(request.params.next().is_none());
    }
    assert!(bytes.is_empty());
}

#[test]
fn mismatched_responses_remain_available_and_remote_errors_are_preserved() {
    let pending = Pending::<sys::functions::NvimEval>::new(MessageId(1));
    let response = Response::<&[u8]> {
        msgid: MessageId(2),
        result: Ok(Object::U8(42)),
    };
    let Err(ResponseError::UnexpectedId { expected, response }) =
        pending.decode::<_, i64>(response)
    else {
        panic!("expected unmatched response");
    };
    assert_eq!(expected, MessageId(1));
    let value: i64 = Pending::<sys::functions::NvimEval>::new(MessageId(2))
        .decode(response)
        .unwrap();
    assert_eq!(value, 42);

    let response = Response::<&[u8]> {
        msgid: MessageId(1),
        result: Err(Object::I8(-1)),
    };
    assert!(matches!(
        pending.decode::<_, i64>(response),
        Err(ResponseError::Remote(Object::I8(-1)))
    ));
}

#[test]
fn notifications_do_not_consume_request_ids() {
    let mut nvim = Neovim::new(Vec::<u8>::new());
    let first = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    nvim.notify::<api::NvimSetCurrentLine>(("notification",))
        .unwrap();
    let second = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    assert_eq!(first.id(), MessageId(0));
    assert_eq!(second.id(), MessageId(1));

    let bytes = nvim.into_inner();
    let mut bytes = &bytes[..];
    assert!(matches!(
        nvim_api::rpc::decode(&mut bytes).unwrap(),
        Message::Request(_)
    ));
    assert!(matches!(
        nvim_api::rpc::decode(&mut bytes).unwrap(),
        Message::Notification(_)
    ));
    assert!(matches!(
        nvim_api::rpc::decode(&mut bytes).unwrap(),
        Message::Request(_)
    ));
    assert!(bytes.is_empty());
}

#[test]
fn partial_synchronous_writes_prevent_further_sends() {
    let mut storage = [0; 1];
    let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut storage[..])));
    assert!(matches!(
        nvim.request::<api::NvimGetCurrentLine>(()),
        Err(SendError::Write(e)) if e.kind() == io::ErrorKind::WriteZero
    ));
    assert_eq!(nvim.writer().get_ref().get_ref()[0], 0x94);
    assert!(nvim.is_failed());
    assert!(matches!(
        nvim.notify::<api::NvimSetCurrentLine>(("later",)),
        Err(SendError::Failed)
    ));
}

#[derive(Clone, Copy)]
enum Mode {
    Ready,
    Pending,
    Error,
}

struct Writer {
    mode: Mode,
    bytes: [u8; 128],
    len: usize,
}

impl Writer {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            bytes: [0; 128],
            len: 0,
        }
    }
}

struct Write<'a> {
    writer: &'a mut Writer,
    bytes: &'a [u8],
}

impl Future for Write<'_> {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { writer, bytes } = self.get_mut();
        match writer.mode {
            Mode::Ready => {
                writer.bytes[writer.len..writer.len + bytes.len()].copy_from_slice(bytes);
                writer.len += bytes.len();
                Poll::Ready(Ok(()))
            }
            Mode::Pending => {
                if let Some(&byte) = bytes.first() {
                    writer.bytes[writer.len] = byte;
                    writer.len += 1;
                }
                Poll::Pending
            }
            Mode::Error => Poll::Ready(Err(io::ErrorKind::BrokenPipe.into())),
        }
    }
}

impl WriterAsync for Writer {
    type Result = io::Result<()>;
    async fn write_all(&mut self, bytes: &[u8]) -> Self::Result {
        Write {
            writer: self,
            bytes,
        }
        .await
    }

    async fn flush(&mut self) -> Self::Result {
        Ok(())
    }
}

#[test]
fn native_async_writer_is_driven_by_the_caller() {
    let mut nvim = Neovim::new(Writer::new(Mode::Ready));
    let pending =
        futures::executor::block_on(nvim.request_async::<api::NvimGetCurrentLine>(())).unwrap();
    assert_eq!(pending.id(), MessageId(0));
    let writer = nvim.into_inner();
    let mut bytes = &writer.bytes[..writer.len];
    let Message::Request(request) = nvim_api::rpc::decode(&mut bytes).unwrap() else {
        panic!("expected request");
    };
    assert_eq!(request.method.as_slice(), b"nvim_get_current_line");
    assert!(bytes.is_empty());
}

#[test]
fn sending_and_flushing_futures_preserve_send() {
    fn assert_send(_: impl Future + Send) {}

    let mut nvim = Neovim::new(Writer::new(Mode::Ready));
    assert_send(nvim.request_async::<api::NvimGetCurrentLine>(()));
    assert_send(nvim.flush_async());

    let mut writer = Writer::new(Mode::Ready);
    let mut nvim = Neovim::new(&mut writer);
    assert_send(nvim.request_async::<api::NvimGetCurrentLine>(()));
    assert_send(nvim.flush_async());
}

#[test]
fn asynchronous_raw_responses_and_notifications_remain_send() {
    use nvim_api::mp::encode::rpc::{Notification, Response};

    fn assert_send(_: impl Future + Send) {}

    let mut writer = Writer::new(Mode::Ready);
    let mut nvim = Neovim::new(&mut writer);
    let response = Response {
        msgid: MessageId(3),
        result: Ok::<_, &str>(42i64),
    };
    assert_send(nvim.send_async(response.to_primitive()));
    let notification = Notification {
        method: "client_event",
        params: ("borrowed",),
    };
    assert_send(nvim.send_async(notification.to_primitive()));
}

#[test]
fn response_sends_preserve_request_ids_and_write_failures() {
    use nvim_api::mp::encode::rpc::Response;

    let mut nvim = Neovim::new(Vec::new());
    let first = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    nvim.send(
        Response {
            msgid: MessageId(99),
            result: Ok::<_, &str>(42u8),
        }
        .to_primitive(),
    )
    .unwrap();
    let next = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    assert_eq!(first.id(), MessageId(0));
    assert_eq!(next.id(), MessageId(1));
    assert!(!nvim.is_failed());

    let mut bytes = [0; 1];
    let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut bytes[..])));
    assert!(matches!(
        nvim.send(Response {
            msgid: MessageId(99),
            result: Ok::<_, &str>(42u8),
        }.to_primitive()),
        Err(SendError::Write(e)) if e.kind() == io::ErrorKind::WriteZero
    ));
    assert!(nvim.is_failed());
}

#[test]
fn asynchronous_response_sends_preserve_failure_and_cancellation_policy() {
    use nvim_api::mp::encode::rpc::Response;

    let mut nvim = Neovim::new(Writer::new(Mode::Error));
    assert!(matches!(
        futures::executor::block_on(nvim.send_async(Response {
            msgid: MessageId(99),
            result: Ok::<_, &str>(42u8),
        }.to_primitive())),
        Err(SendError::Write(e)) if e.kind() == io::ErrorKind::BrokenPipe
    ));
    assert!(nvim.is_failed());

    let mut nvim = Neovim::new(Writer::new(Mode::Pending));
    drop(
        nvim.send_async(
            Response {
                msgid: MessageId(99),
                result: Ok::<_, &str>(42u8),
            }
            .to_primitive(),
        ),
    );
    assert!(!nvim.is_failed());
    {
        let mut future = std::pin::pin!(
            nvim.send_async(
                Response {
                    msgid: MessageId(99),
                    result: Ok::<_, &str>(42u8),
                }
                .to_primitive()
            )
        );
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }
    assert!(nvim.is_failed());
    assert_eq!(nvim.writer().len, 1);
    assert!(matches!(
        futures::executor::block_on(
            nvim.send_async(
                Response {
                    msgid: MessageId(99),
                    result: Ok::<_, &str>(42u8),
                }
                .to_primitive()
            )
        ),
        Err(SendError::Failed)
    ));
}

#[test]
fn cancelling_a_polled_send_marks_the_sender_failed() {
    let mut nvim = Neovim::new(Writer::new(Mode::Pending));
    drop(nvim.request_async::<api::NvimGetCurrentLine>(()));
    assert!(!nvim.is_failed());

    {
        let mut future = std::pin::pin!(nvim.request_async::<api::NvimGetCurrentLine>(()));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }
    assert!(nvim.is_failed());
    assert_eq!(nvim.writer().len, 1);
    assert!(matches!(
        futures::executor::block_on(nvim.request_async::<api::NvimGetCurrentLine>(())),
        Err(SendError::Failed)
    ));
}

#[test]
fn asynchronous_write_errors_are_reported_and_prevent_further_sends() {
    let mut nvim = Neovim::new(Writer::new(Mode::Error));
    assert!(matches!(
        futures::executor::block_on(nvim.request_async::<api::NvimGetCurrentLine>(())),
        Err(SendError::Write(e)) if e.kind() == io::ErrorKind::BrokenPipe
    ));
    assert!(matches!(
        futures::executor::block_on(nvim.notify_async::<api::NvimGetCurrentLine>(())),
        Err(SendError::Failed)
    ));
}
