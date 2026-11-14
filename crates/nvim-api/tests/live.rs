use nvim_api::api_types::{Decode, Params};
use nvim_api::mp::decode::{Array, Object};
use nvim_api::mp::encode::primitive::Nil;
use nvim_api::mp::encode::primitive::State;
use nvim_api::mp::encode::{Primitive, StdWrite};
use nvim_api::read::StdRead;
use nvim_api::rpc::Message;
use nvim_api::sys::types::ApiFn;
use nvim_api::{Buffer, Neovim, Receiver, UiOptions, api, ui, ui_events};

use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Session {
    nvim: Neovim<StdWrite<ChildStdin>>,
    receiver: Receiver<StdRead<ChildStdout>, Vec<u8>>,
    child: Child,
}

impl Session {
    fn new() -> Self {
        let bin = std::env::var_os("NVIM_BIN").unwrap_or_else(|| "nvim".into());
        let mut child = Command::new(bin)
            .args(["--embed", "--headless", "--clean", "-n", "-i", "NONE"])
            .env("NVIM_LOG_FILE", "/tmp/nvim-api-integration.log")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("could not start Neovim; set NVIM_BIN to its executable");
        let nvim = Neovim::new(StdWrite::new(child.stdin.take().unwrap()));
        let receiver = Receiver::new(StdRead(child.stdout.take().unwrap()), vec![0; 1024 * 1024]);
        Self {
            nvim,
            receiver,
            child,
        }
    }

    fn call<F, T>(
        &mut self,
        params: impl Params<F::Params, Encoded: Send + Primitive<State: State<Item: Send> + Send>>,
    ) -> T
    where
        F: ApiFn,
        for<'a> T: Decode<F::Return, &'a [u8]>,
    {
        let pending = self.nvim.request::<F>(params).unwrap();
        loop {
            match self.receiver.recv().unwrap() {
                Message::Response(response) if pending.matches(&response) => {
                    return pending.decode(response).unwrap();
                }
                Message::Notification(_) => {}
                _ => panic!("unexpected RPC message"),
            }
        }
    }

    fn channel_id(&mut self) -> i64 {
        let pending = self.nvim.request::<api::NvimGetApiInfo>(()).unwrap();
        loop {
            match self.receiver.recv().unwrap() {
                Message::Response(response) if pending.matches(&response) => {
                    let mut info: Array<&[u8]> = pending.decode(response).unwrap();
                    return <i64 as Decode<nvim_api::sys::types::Object, &[u8]>>::decode(
                        info.next().unwrap(),
                    )
                    .unwrap();
                }
                Message::Notification(_) => {}
                _ => panic!("unexpected API metadata response"),
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
#[ignore = "requires a local Neovim executable; set NVIM_BIN"]
fn command_eval_buffer_lines_and_byte_strings_round_trip() {
    let mut session = Session::new();
    let (): () = session.call::<api::NvimCommand, _>(("let g:nvim_api_test = 40",));
    let result: i64 = session.call::<api::NvimEval, _>(("g:nvim_api_test + 2",));
    assert_eq!(result, 42);

    let buffer: Buffer = session.call::<api::NvimCreateBuf, _>((false, true));
    let lines = ["borrowed line", "日本語"];
    let (): () = session.call::<api::NvimBufSetLines, _>((buffer, 0, -1, true, &lines[..]));
    let actual: Vec<String> = session.call::<api::NvimBufGetLines, _>((buffer, 0, -1, true));
    assert_eq!(actual, lines);

    let bytes = &[0xff, b'x', 0x80][..];
    let (): () = session.call::<api::NvimSetCurrentLine, _>((bytes,));
    let pending = session.nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
    let Message::Response(response) = session.receiver.recv().unwrap() else {
        panic!("expected byte-string response");
    };
    let actual: &[u8] = pending.decode(response).unwrap();
    assert_eq!(actual, bytes);
    assert!(std::str::from_utf8(actual).is_err());
}

#[test]
#[ignore = "requires a local Neovim executable; set NVIM_BIN"]
fn notifications_and_nested_plugin_requests_round_trip() {
    let mut session = Session::new();
    let channel = session.channel_id();
    let pending = session
        .nvim
        .request::<api::NvimExecLua>((
            "local chan = ...; vim.rpcnotify(chan, 'client_event', 'hello'); return 7",
            (channel,),
        ))
        .unwrap();
    let mut notification_received = false;
    let mut response_received = false;
    while !notification_received || !response_received {
        match session.receiver.recv().unwrap() {
            Message::Notification(mut notification) => {
                assert_eq!(notification.method.as_slice(), b"client_event");
                let Some(Object::Str(text)) = notification.params.next() else {
                    panic!("expected notification argument");
                };
                assert_eq!(text.as_slice(), b"hello");
                notification_received = true;
            }
            Message::Response(response) => {
                assert_eq!(pending.decode::<_, i64>(response).unwrap(), 7);
                response_received = true;
            }
            _ => panic!("unexpected notification test message"),
        }
    }

    let pending = session
        .nvim
        .request::<api::NvimExecLua>((
            "local chan = ...; return vim.rpcrequest(chan, 'client_double', 21)",
            (channel,),
        ))
        .unwrap();
    let Message::Request(mut request) = session.receiver.recv().unwrap() else {
        panic!("expected nested plugin request");
    };
    assert_eq!(request.method.as_slice(), b"client_double");
    let value = <i64 as Decode<nvim_api::sys::types::Object, &[u8]>>::decode(
        request.params.next().unwrap(),
    )
    .unwrap();
    session
        .nvim
        .send(
            nvim_api::mp::encode::rpc::Response {
                msgid: request.msgid,
                result: Ok::<_, Nil>(value * 2),
            }
            .to_primitive(),
        )
        .unwrap();
    let Message::Response(response) = session.receiver.recv().unwrap() else {
        panic!("expected Lua result after plugin response");
    };
    assert_eq!(pending.decode::<_, i64>(response).unwrap(), 42);
}

#[test]
#[ignore = "requires a local Neovim executable; set NVIM_BIN"]
fn ui_attach_produces_borrowed_grid_resize_and_flush_events() {
    let mut session = Session::new();
    let options = UiOptions {
        ext_linegrid: true,
        ..UiOptions::default()
    };
    let attach = session
        .nvim
        .request::<api::NvimUiAttach>((20, 6, options))
        .unwrap();
    let redraw = session
        .nvim
        .request::<api::NvimCommand>(("redraw",))
        .unwrap();
    let mut attached = false;
    let mut resized = false;
    let mut flushed = false;
    let mut redrawn = false;
    while !attached || !resized || !flushed || !redrawn {
        match session.receiver.recv().unwrap() {
            Message::Response(response) if attach.matches(&response) => {
                attach.decode::<_, ()>(response).unwrap();
                attached = true;
            }
            Message::Response(response) if redraw.matches(&response) => {
                redraw.decode::<_, ()>(response).unwrap();
                redrawn = true;
            }
            Message::Notification(notification) => {
                for event in ui::Redraw::new(notification).unwrap() {
                    let mut event = event.unwrap();
                    if event.is::<ui_events::GridResize>() {
                        while let Some((grid, width, height)) = event
                            .decode_next::<ui_events::GridResize, (i64, i64, i64)>()
                            .unwrap()
                        {
                            if grid == 1 {
                                assert_eq!((width, height), (20, 6));
                                resized = true;
                            }
                        }
                    } else if event.is::<ui_events::Flush>() {
                        assert_eq!(
                            event.decode_next::<ui_events::Flush, ()>().unwrap(),
                            Some(())
                        );
                        flushed = true;
                    }
                }
            }
            _ => panic!("unexpected UI message"),
        }
    }
}
