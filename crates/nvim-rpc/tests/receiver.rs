use nvim_rpc::read::{ReaderAsync, ReaderSync, StdRead};
use nvim_rpc::rpc::Message;
use nvim_rpc::{ReadError, Receiver};

use std::future::{Future, poll_fn};
use std::io;
use std::task::{Context, Poll, Waker};

const TWO_MESSAGES: &[u8] = &[0x93, 2, 0xa1, b'a', 0x90, 0x93, 2, 0xa1, b'b', 0x90];

fn assert_notification(message: Message<&[u8]>, method: &[u8]) {
    let Message::Notification(notification) = message else {
        panic!("expected notification");
    };
    assert_eq!(notification.method.as_slice(), method);
    assert_eq!(notification.params.len(), 0);
}

struct Fragmented<'a> {
    bytes: &'a [u8],
    chunk: usize,
}

impl ReaderSync for Fragmented<'_> {
    type Error = io::Error;

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.bytes.len().min(self.chunk).min(buf.len());
        buf[..len].copy_from_slice(&self.bytes[..len]);
        self.bytes = &self.bytes[len..];
        Ok(len)
    }
}

#[test]
fn reads_fragmented_and_concatenated_messages_with_fixed_storage() {
    for chunk in [1, 2, 7, 10] {
        let reader = Fragmented {
            bytes: TWO_MESSAGES,
            chunk,
        };
        let mut receiver = Receiver::new(reader, [0; 7]);
        assert_notification(receiver.recv().unwrap(), b"a");
        assert_notification(receiver.recv().unwrap(), b"b");
        assert!(matches!(
            receiver.recv(),
            Err(ReadError::Eof { buffered: 0 })
        ));
    }
}

#[test]
fn callers_can_replace_full_storage_without_losing_partial_frames() {
    let mut receiver = Receiver::new(StdRead(TWO_MESSAGES), [0; 4]);
    assert!(matches!(
        receiver.recv(),
        Err(ReadError::BufferFull { capacity: 4 })
    ));
    assert_eq!(receiver.buffered(), &TWO_MESSAGES[..4]);
    let receiver = match receiver.with_storage([0; 3]) {
        Err(receiver) => receiver,
        Ok(_) => panic!("undersized storage should be rejected"),
    };
    let mut receiver = match receiver.with_storage([0; 8]) {
        Ok(receiver) => receiver,
        Err(_) => panic!("larger storage should be accepted"),
    };
    assert_notification(receiver.recv().unwrap(), b"a");
    assert_notification(receiver.recv().unwrap(), b"b");
}

#[test]
fn eof_distinguishes_incomplete_input() {
    let mut receiver = Receiver::new(StdRead(&TWO_MESSAGES[..4]), [0; 8]);
    assert!(matches!(
        receiver.recv(),
        Err(ReadError::Eof { buffered: 4 })
    ));
    assert_eq!(receiver.buffered(), &TWO_MESSAGES[..4]);
}

struct AsyncReader<'a> {
    bytes: &'a [u8],
    reads: usize,
    pause: bool,
    fail: Option<usize>,
}

impl ReaderAsync for AsyncReader<'_> {
    type Error = io::Error;

    async fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> io::Result<usize> {
        poll_fn(|_| {
            if self.fail == Some(self.reads) {
                self.fail = None;
                return Poll::Ready(Err(io::ErrorKind::Interrupted.into()));
            }
            if self.reads == 1 && self.pause {
                self.pause = false;
                return Poll::Pending;
            }
            self.reads += 1;
            let len = self.bytes.len().min(2).min(buf.len());
            buf[..len].copy_from_slice(&self.bytes[..len]);
            self.bytes = &self.bytes[len..];
            Poll::Ready(Ok(len))
        })
        .await
    }
}

#[test]
fn cancelling_receive_preserves_framing_progress() {
    let reader = AsyncReader {
        bytes: TWO_MESSAGES,
        reads: 0,
        pause: true,
        fail: None,
    };
    let mut receiver = Receiver::new(reader, [0; 8]);
    {
        let mut future = std::pin::pin!(receiver.recv_async());
        let mut cx = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }
    assert_eq!(receiver.buffered(), &TWO_MESSAGES[..2]);
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"a",
    );
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"b",
    );
}

#[test]
fn reader_errors_leave_the_receiver_available_for_retry() {
    let reader = AsyncReader {
        bytes: TWO_MESSAGES,
        reads: 0,
        pause: false,
        fail: Some(0),
    };
    let mut receiver = Receiver::new(reader, [0; 8]);
    assert!(matches!(
        futures::executor::block_on(receiver.recv_async()),
        Err(ReadError::Read(e)) if e.kind() == io::ErrorKind::Interrupted
    ));
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"a",
    );
}

#[test]
fn asynchronous_receives_can_be_spawned_when_the_reader_is_send() {
    fn assert_send(_: impl Future + Send) {}

    let reader = AsyncReader {
        bytes: TWO_MESSAGES,
        reads: 0,
        pause: false,
        fail: None,
    };
    let mut receiver = Receiver::new(reader, [0; 8]);
    assert_send(receiver.recv_async());

    let mut reader = AsyncReader {
        bytes: TWO_MESSAGES,
        reads: 0,
        pause: false,
        fail: None,
    };
    let mut receiver = Receiver::new(&mut reader, [0; 8]);
    assert_send(receiver.recv_async());
}

#[test]
fn reader_errors_preserve_partially_read_frames() {
    let reader = AsyncReader {
        bytes: TWO_MESSAGES,
        reads: 0,
        pause: false,
        fail: Some(1),
    };
    let mut receiver = Receiver::new(reader, [0; 8]);
    assert!(matches!(
        futures::executor::block_on(receiver.recv_async()),
        Err(ReadError::Read(e)) if e.kind() == io::ErrorKind::Interrupted
    ));
    assert_eq!(receiver.buffered(), &TWO_MESSAGES[..2]);
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"a",
    );
}

#[cfg(feature = "tokio")]
#[test]
fn tokio_read_adapter_preserves_send_and_borrowed_messages() {
    fn assert_send(_: impl Future + Send) {}

    let mut receiver = Receiver::new(nvim_rpc::read::TokioRead(TWO_MESSAGES), [0; 8]);
    assert_send(receiver.recv_async());
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"a",
    );
}

#[cfg(feature = "futures")]
#[test]
fn futures_read_adapter_preserves_send_and_borrowed_messages() {
    fn assert_send(_: impl Future + Send) {}

    let mut receiver = Receiver::new(nvim_rpc::read::FuturesRead(TWO_MESSAGES), [0; 8]);
    assert_send(receiver.recv_async());
    assert_notification(
        futures::executor::block_on(receiver.recv_async()).unwrap(),
        b"a",
    );
}
