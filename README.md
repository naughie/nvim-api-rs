# nvim-api

Rust libraries for communicating with Neovim over MessagePack-RPC, for plugins,
UI clients, and programs that control an editor.

Arguments can be borrowed slices, owned values, or iterators. Results can borrow
the input buffer or be decoded into owned Rust values. The core crates let the
caller supply the transport, receive storage, and event loop. `nvim-api-plugin`
provides a Tokio layer with owned messages and spawned handlers.

## Crates

| Crate | What it provides |
| --- | --- |
| [nvim-api](crates/nvim-api) | Typed API descriptors, handles, result decoding, an RPC sender and receiver, and redraw event decoding |
| [nvim-api-plugin](crates/nvim-api-plugin) | Tokio plugin handlers and a cloneable client that waits for responses |
| [nvim-api-sys](crates/nvim-api-sys) | Generated Neovim function signatures, UI event signatures, type markers, and API version metadata |
| [rmp-bytes](crates/rmp-bytes) | MessagePack encoding and decoding over caller-selected writers and buffers |
| [codegen](codegen) | The tool that regenerates the Neovim API bindings |

## Getting started

Start with [nvim-api-plugin](crates/nvim-api-plugin/README.md). Implement its
`Handler<W>` trait for your RPC methods; it runs the receive loop, matches
responses, and lets a handler call back into Neovim while awaiting a reply.

The [stdio example](crates/nvim-api-plugin/examples/plugin.rs) handles an `add`
request and a `set_line` notification. Its custom request and response types
show where to implement `FromParams` and `IntoResponse`. Build it with:

```sh
cargo build -p nvim-api-plugin --example plugin
```

The plugin README includes the Lua calls to launch and use the example.

For applications that manage their own receive loop, start with the
[nvim-api guide](crates/nvim-api/README.md). It shows borrowed arguments,
response decoding, custom argument types, and optional async I/O adapters.

## Try it against Neovim

The [sync example](crates/nvim-api/examples/sync.rs) starts an embedded headless
Neovim, evaluates an expression, reads a borrowed string response, and exits:

```sh
cargo run -p nvim-api --example sync
```

Set `NVIM_BIN` if Neovim is not on `PATH`:

```sh
NVIM_BIN=/path/to/nvim cargo run -p nvim-api --example sync
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for design constraints, validation, and
binding regeneration.
