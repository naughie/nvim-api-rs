# nvim-rpc

[![crates.io](https://img.shields.io/crates/v/nvim-rpc.svg)](https://crates.io/crates/nvim-rpc)
[![docs.rs](https://docs.rs/nvim-rpc/badge.svg)](https://docs.rs/nvim-rpc)

Typed Neovim RPC calls with caller-selected argument types, result types, and
I/O. Pass `&str`, `&[&str]`, owned values, or exact-size iterators without
collecting them into a prescribed set of containers.

The crate builds and writes calls, reads messages, and decodes results. Your
application owns the connection, response dispatch, and task scheduling.

For a Tokio plugin with `Handler` callbacks and a client that waits for replies,
start with [nvim-rpc-plugin](../nvim-rpc-plugin). Use this crate directly when
your application supplies its own receive loop, runtime, or storage.

## Write a call and read its result

This example writes into a fixed buffer and reads a supplied MessagePack
response. Replace the cursor and input slice with process pipes or a socket
when connecting to Neovim.

```rust
use nvim_rpc::mp::encode::StdWrite;
use nvim_rpc::read::StdRead;
use nvim_rpc::rpc::Message;
use nvim_rpc::{api, Buffer, Neovim, Receiver};
use std::io::Cursor;

let mut output = [0; 512];
let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut output[..])));
let lines = ["first", "second"];
nvim.notify::<api::NvimBufSetLines>((Buffer::CURRENT, 0i64, -1i64, true, &lines[..]))
    .unwrap();

let pending = nvim.request::<api::NvimGetCurrentLine>(()).unwrap();
nvim.flush().unwrap();
let response_bytes = &b"\x94\x01\x00\xc0\xa5hello"[..];
let mut receiver = Receiver::new(StdRead(response_bytes), [0; 64]);
let Message::Response(response) = receiver.recv().unwrap() else {
    panic!("expected response");
};
let line: &str = pending.decode(response).unwrap();
assert_eq!(line, "hello");
```

`api::NvimGetCurrentLine` selects the API function. `()` means no arguments;
a one-argument call uses `(value,)`. `request` returns after writing the request.
Its `Pending` value matches the response ID and selects the return-type marker.
Choose the Rust result type at `pending.decode(response)`. Dispatch incoming
requests and notifications in your receive loop while awaiting the matching
response.

Here, `line` borrows the receiver's storage. Decode into `String` to retain it
across later receives. Arrays can likewise be decoded lazily with
`DecodedArray`, or collected into a chosen `Vec<T>`.

For a runnable connection to a local Neovim process, see
[`examples/sync.rs`](examples/sync.rs).

## Tokio

Enable `nvim-rpc`'s `tokio` feature and Tokio's `io-util` feature. Wrap an
existing writer with `TokioWrite`; the caller supplies and runs the runtime.

```rust
use nvim_rpc::mp::encode::TokioWrite;
use nvim_rpc::{api, Neovim};
use tokio::io::AsyncWrite;

async fn set_line<W: AsyncWrite + Unpin>(writer: W)
    -> Result<(), Box<dyn std::error::Error>>
{
    let mut nvim = Neovim::new(TokioWrite::new(writer));
    nvim.notify_async::<api::NvimSetCurrentLine>(("hello from Rust",))
        .await?;
    nvim.flush_async().await?;
    Ok(())
}
```

The matching reader is `read::TokioRead`, used with `Receiver::recv_async`.
Use `flush()` or `flush_async()` before waiting for a reply. The crate does not
spawn tasks or maintain a request registry.

## Features and extension points

| Feature | Adds |
| --- | --- |
| none | `std::io` adapters and the transport-independent traits |
| `tokio` | Tokio reader and writer adapters |
| `futures` | futures I/O reader and writer adapters |
| `bytes` | `bytes` buffer support through `rmp-bytes` |

Implement `api_types::encode::Like<Marker>` for application arguments and
`api_types::decode::Decode<Marker, B>` for application results. See the
`api_types` module documentation for examples of both.

Typed call conversion requires `Send` encoded values, states, and chunks.
Custom transports may still be local, and their futures need not be `Send`.
A failed or cancelled write can leave a partial RPC frame on the connection;
the sender rejects later sends after an interrupted write.

The crate documentation also covers plugin replies, receive-buffer replacement,
and UI redraw decoding. Build it locally with `cargo doc -p nvim-rpc --open`.
