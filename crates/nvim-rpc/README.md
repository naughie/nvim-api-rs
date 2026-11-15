# nvim-rpc

[![crates.io](https://img.shields.io/crates/v/nvim-rpc.svg)](https://crates.io/crates/nvim-rpc)
[![docs.rs](https://docs.rs/nvim-rpc/badge.svg)](https://docs.rs/nvim-rpc)

Typed Neovim RPC calls with caller-selected arguments, results, and I/O.

The crate builds and writes calls, reads messages, and decodes results. Your
application owns the connection, response dispatch, and task scheduling.

For a Tokio plugin with `Handler` callbacks and a client that waits for replies,
start with [nvim-rpc-plugin](../nvim-rpc-plugin). Use this crate directly when
your application supplies its own receive loop, runtime, or storage.

## Getting started

This example writes into a fixed buffer and reads a supplied MessagePack
response. Replace the cursor and input slice with process pipes or a socket
when connecting to Neovim.

```rust
use nvim_rpc::mp::encode::StdWrite;
use nvim_rpc::read::StdRead;
use nvim_rpc::rpc::Message;
use nvim_rpc::{api, Neovim, Receiver};
use std::io::Cursor;

let mut output = [0; 512];
let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut output[..])));
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

For a runnable connection to a local Neovim process, see
[`examples/sync.rs`](examples/sync.rs).

## Features

| Feature | Adds |
| --- | --- |
| none | `std::io` adapters and the transport-independent traits |
| `tokio` | Tokio reader and writer adapters |
| `futures` | futures I/O reader and writer adapters |
| `bytes` | `bytes` buffer support through `rmp-bytes` |

See the [API documentation](https://docs.rs/nvim-rpc/latest/nvim_rpc/) for
transport setup, request and response handling, custom representations, and UI
notifications. Development and validation procedures are in
[CONTRIBUTING.md](../../CONTRIBUTING.md).
