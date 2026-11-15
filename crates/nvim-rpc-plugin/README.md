# nvim-rpc-plugin

[![crates.io](https://img.shields.io/crates/v/nvim-rpc-plugin.svg)](https://crates.io/crates/nvim-rpc-plugin)
[![docs.rs](https://docs.rs/nvim-rpc-plugin/badge.svg)](https://docs.rs/nvim-rpc-plugin)

Tokio plugin handlers and a client for calling Neovim. `spawn` runs the receive
loop, dispatches requests and notifications, and matches replies to waiting
calls. Handlers can call Neovim while processing an incoming request.

This crate owns message buffers and starts Tokio tasks. Use
[`nvim-rpc`](../nvim-rpc) to manage buffers and dispatch in your application.

## Run a stdio plugin

The [example](examples/plugin.rs) handles an `add` request and a `set_line`
notification. It defines custom argument and response types, and calls back into
Neovim to read the current line while handling `add`:

```sh
cargo build -p nvim-rpc-plugin --example plugin
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
```

Stdout carries RPC bytes; write diagnostics to stderr.

## Use the crate

Implement `Handler<W>` and call `spawn(from_neovim, to_neovim, handler)` within
a Tokio runtime. The [stdio example](examples/plugin.rs) shows custom arguments,
responses, and calls back into Neovim.

See the API reference for:

- [`Handler`](https://docs.rs/nvim-rpc-plugin/latest/nvim_rpc_plugin/trait.Handler.html):
  callbacks and their execution order.
- [`FromParams`](https://docs.rs/nvim-rpc-plugin/latest/nvim_rpc_plugin/trait.FromParams.html)
  and [`IntoResponse`](https://docs.rs/nvim-rpc-plugin/latest/nvim_rpc_plugin/trait.IntoResponse.html):
  argument and response representations.
- [`Neovim`](https://docs.rs/nvim-rpc-plugin/latest/nvim_rpc_plugin/struct.Neovim.html)
  and [`spawn`](https://docs.rs/nvim-rpc-plugin/latest/nvim_rpc_plugin/fn.spawn.html):
  API calls and connection lifecycle.

Build the documentation locally with `cargo doc -p nvim-rpc-plugin --open`.
Contributor procedures are in [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Acknowledgement

The `Handler` and `Neovim` API is inspired by
[nvim-rs](https://github.com/KillTheMule/nvim-rs), by KillTheMule and its
contributors.
