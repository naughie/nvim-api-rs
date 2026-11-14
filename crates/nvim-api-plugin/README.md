# nvim-api-plugin

[![crates.io](https://img.shields.io/crates/v/nvim-api-plugin.svg)](https://crates.io/crates/nvim-api-plugin)
[![docs.rs](https://docs.rs/nvim-api-plugin/badge.svg)](https://docs.rs/nvim-api-plugin)

Tokio plugin handlers and a client for calling Neovim. `spawn` runs the receive
loop, dispatches requests and notifications, and matches replies to waiting
calls. Handlers can call Neovim while processing an incoming request.

This crate owns message buffers and starts Tokio tasks. Use
[`nvim-api`](../nvim-api) to manage buffers and dispatch in your application.

## Run a stdio plugin

The [example](examples/plugin.rs) handles an `add` request and a `set_line`
notification. It defines custom argument and response types, and calls back into
Neovim to read the current line while handling `add`:

```sh
cargo build -p nvim-api-plugin --example plugin
```

From the workspace root, start Neovim and run this Lua. Use an absolute executable
path if Neovim's current directory differs:

```lua
local channel = vim.fn.jobstart({ vim.fn.getcwd() .. "/target/debug/examples/plugin" }, {
  rpc = true,
  on_stderr = vim.schedule_wrap(function(_, data)
    local text = table.concat(data, "\n")
    if text ~= "" then vim.notify(text) end
  end),
})
assert(channel > 0, "could not start plugin")
print(vim.rpcrequest(channel, "add", 20, 22, "hello")) -- 42
vim.rpcnotify(channel, "set_line", "written by Rust")
local ok, err = pcall(vim.rpcrequest, channel, "add", 2147483647, 1, "overflow")
assert(not ok)
print(err)
```

Stdout carries RPC bytes; write diagnostics to stderr.

## Define handlers

Implement `Handler<W>` with `async fn` callbacks. Its `Request` and
`Notification` types implement `FromParams`, and its `Response` type implements
`IntoResponse`.

Tuples decode a fixed argument layout and require its exact length. For example,
`Request = (i32, i32, Bin)` checks two integer ranges and retains the third
argument's bytes. Use `Array` when methods have different layouts: match the
method name in the callback, then decode the appropriate tuple. `()` accepts
only an empty argument array.

`Bin`, `Array`, `Map`, and `Object` retain shared message storage across receives
and handler tasks. `String` validates UTF-8 and copies the text into an owned
string. Custom argument types can delegate to a tuple's `FromParams`
implementation, as the example's `Add` type does.

A `Result<T, E>` response encodes its success or error value. Custom response
types select those values through `IntoResponse`, as `AddResponse` does in the
example. `()` produces a successful empty array; use `mp::encode::primitive::Nil`
as the success value of a `Result` response for nil.

## Call Neovim

Select a generated API descriptor and supply an argument tuple. The second
`request` type parameter selects the Rust result representation:

```rust
use nvim_api_plugin::{Bin, Neovim, api};
use tokio::io::AsyncWrite;

async fn current_line<W: AsyncWrite + Unpin + Send + 'static>(
    nvim: &Neovim<W>,
) -> Result<Bin, nvim_api_plugin::Error> {
    nvim.request::<api::NvimGetCurrentLine, Bin>(()).await
}
```

`request` writes, flushes, waits, and decodes the matching reply. `notify` writes
and flushes without waiting for a reply:

```rust
use nvim_api_plugin::{Neovim, api};
use tokio::io::AsyncWrite;

async fn set_line<W: AsyncWrite + Unpin + Send + 'static>(
    nvim: &Neovim<W>,
) -> Result<(), nvim_api_plugin::Error> {
    nvim.notify::<api::NvimSetCurrentLine>(("hello from Rust",)).await
}
```

Clone the client to make concurrent calls. Each call releases the shared writer
before waiting for its reply.

## Manage the connection

Call `spawn(from_neovim, to_neovim, handler)` within a Tokio runtime. A stdio
plugin passes stdin and stdout; a program starting an embedded Neovim passes the
child's stdout and stdin. Pass the raw writer: the connection buffers encoded
chunks and flushes each complete request, notification, and reply.

`spawn` returns a client and a connection `JoinHandle`. Await the handle to
observe EOF, transport errors, invalid notification arguments, or handler panics.
Aborting it closes the connection, stops handler tasks, and wakes waiting calls.
Notifications can run concurrently; request replies are sent in reverse request
order.

Dropping a request future while it waits for a reply leaves the remote operation
running; its late reply is consumed. Cancelling a write closes the connection.

## Acknowledgement

The `Handler` and `Neovim` API is inspired by
[nvim-rs](https://github.com/KillTheMule/nvim-rs), by KillTheMule and its
contributors.
