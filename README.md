# nvim-rpc

Rust libraries for communicating with Neovim over MessagePack-RPC, for plugins,
UI clients, and programs that control an editor.

Arguments can be borrowed slices, owned values, or iterators. Results can borrow
the input buffer or be decoded into owned Rust values. The core crates let the
caller supply the transport, receive storage, and event loop. `nvim-rpc-plugin`
provides Tokio RPC dispatch and a client that waits for responses.
`nvim-rpc-ui` adds a backend trait, UI state, keyboard input, and embedded editor
sessions using the same RPC connection.

## Crates

| Crate | What it provides |
| --- | --- |
| [nvim-rpc](crates/nvim-rpc) | Typed API descriptors, handles, result decoding, an RPC sender and receiver, and redraw event decoding |
| [nvim-rpc-plugin](crates/nvim-rpc-plugin) | Tokio plugin handlers and a cloneable client that waits for responses |
| [nvim-rpc-ui](crates/nvim-rpc-ui) | A backend trait, UI state, keyboard input, and embedded Neovim sessions |
| [nvim-rpc-sys](crates/nvim-rpc-sys) | Generated Neovim function signatures, UI event signatures, type markers, and API version metadata |
| [rmp-bytes](crates/rmp-bytes) | MessagePack encoding and decoding over caller-selected writers and buffers |
| [codegen](codegen) | The tool that regenerates the Neovim API bindings |

## Getting started

Start with [nvim-rpc-plugin](crates/nvim-rpc-plugin/README.md). Implement its
`Handler<W>` trait for your RPC methods; it runs the receive loop, matches
responses, and lets a handler call back into Neovim while awaiting a reply.

The [stdio example](crates/nvim-rpc-plugin/examples/plugin.rs) handles an `add`
request and a `set_line` notification. Its custom request and response types
show where to implement `FromParams` and `IntoResponse`. Build it with:

```sh
cargo build -p nvim-rpc-plugin --example plugin
```

The plugin README includes the Lua calls to launch and use the example.

For applications that manage their own receive loop, start with
[nvim-rpc](crates/nvim-rpc/README.md). Each crate's README links to its API
documentation for detailed usage and contracts.

For a UI frontend, start with [nvim-rpc-ui](crates/nvim-rpc-ui/README.md).
Implement `Backend` for your toolkit and receive frames from an embedded editor
or an existing Neovim connection. Its [API documentation](https://docs.rs/nvim-rpc-ui)
covers drawing, input handling, and sharing a plugin connection.

## Try it against Neovim

The [sync example](crates/nvim-rpc/examples/sync.rs) starts an embedded headless
Neovim, evaluates an expression, reads a borrowed string response, and exits:

```sh
cargo run -p nvim-rpc --example sync
```

Set `NVIM_BIN` if Neovim is not on `PATH`:

```sh
NVIM_BIN=/path/to/nvim cargo run -p nvim-rpc --example sync
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for design constraints, validation, and
binding regeneration.
