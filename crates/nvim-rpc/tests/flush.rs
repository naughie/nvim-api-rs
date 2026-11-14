use nvim_rpc::mp::encode::StdWrite;
use nvim_rpc::rpc::{Message, MessageId};
use nvim_rpc::{Neovim, api};

use std::io::BufWriter;

fn assert_requests(mut bytes: &[u8], ids: &[MessageId]) {
    for &id in ids {
        let Message::Request(mut request) = nvim_rpc::rpc::decode(&mut bytes).unwrap() else {
            panic!("expected request");
        };
        assert_eq!(request.msgid, id);
        assert_eq!(request.method.as_slice(), b"nvim_get_current_line");
        assert!(request.params.next().is_none());
    }
    assert!(bytes.is_empty());
}

#[test]
fn flushes_borrowed_synchronous_writer_without_consuming_request_ids() {
    let mut writer = StdWrite::new(BufWriter::with_capacity(1024, Vec::new()));
    let mut nvim = Neovim::new(&mut writer);
    let first = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    assert_eq!(first.id(), MessageId(0));
    assert!(nvim.writer().get_ref().get_ref().is_empty());

    nvim.flush().unwrap();
    assert_requests(nvim.writer().get_ref().get_ref(), &[first.id()]);
    let len = nvim.writer().get_ref().get_ref().len();

    let second = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    assert_eq!(second.id(), MessageId(1));
    assert_eq!(nvim.writer().get_ref().get_ref().len(), len);

    nvim.flush().unwrap();
    assert_requests(
        nvim.writer().get_ref().get_ref(),
        &[first.id(), second.id()],
    );
}

#[cfg(feature = "tokio")]
#[tokio::test(flavor = "current_thread")]
async fn flushes_tokio_buffered_writer() {
    use nvim_rpc::mp::encode::TokioWrite;

    let writer = TokioWrite::new(tokio::io::BufWriter::with_capacity(1024, Vec::new()));
    let mut nvim = Neovim::new(writer);
    let pending = nvim
        .request_async::<api::NvimGetCurrentLine>(())
        .await
        .unwrap();
    assert!(nvim.writer().get_ref().get_ref().is_empty());

    nvim.flush_async().await.unwrap();
    assert_requests(nvim.writer().get_ref().get_ref(), &[pending.id()]);
}

#[cfg(feature = "futures")]
#[test]
fn flushes_futures_buffered_writer() {
    use nvim_rpc::mp::encode::FuturesWrite;

    use futures::io::{BufWriter, Cursor};

    futures::executor::block_on(async {
        let writer = FuturesWrite::new(BufWriter::with_capacity(1024, Cursor::new(Vec::new())));
        let mut nvim = Neovim::new(writer);
        let pending = nvim
            .request_async::<api::NvimGetCurrentLine>(())
            .await
            .unwrap();
        assert!(nvim.writer().get_ref().get_ref().get_ref().is_empty());

        nvim.flush_async().await.unwrap();
        assert_requests(nvim.writer().get_ref().get_ref().get_ref(), &[pending.id()]);
    });
}
