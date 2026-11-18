# nvim-rpc-ui

Neovim UI integration for Rust applications. Implement `Backend` for your GUI
or terminal toolkit to draw grids, highlights, the cursor, and external widgets.
Connections use the `nvim-rpc-plugin` client and RPC dispatcher.

## Get started

Use a Neovim executable that supports `ext_linegrid` and a running Tokio
runtime with I/O enabled. Run presentation on the thread that owns your
toolkit's widgets. `Backend` has no `Send`, `Sync`, or `'static` requirement.

This function starts an embedded editor and presents frames through an
application-provided backend:

```rust,no_run
# async fn run<B: nvim_rpc_ui::Backend>(mut backend: B)
#     -> Result<(), nvim_rpc_ui::PresentationError<B::Error>> {
use nvim_rpc_ui::{Embedded, PresentationError, UiOptions};

let mut command = tokio::process::Command::new("nvim");
let options = UiOptions { ext_linegrid: true, ..UiOptions::default() };
let mut editor = Embedded::spawn(&mut command, 100, 30, options)
    .await.map_err(PresentationError::Session)?;

while editor.present_next(&mut backend).await? {}
editor.shutdown().await.map_err(PresentationError::Session)?;
# Ok(()) }
```

`Backend::present()` receives an owned `Frame`. Draw its state and process its
events in order. See the [backend example](https://docs.rs/nvim-rpc-ui/latest/nvim_rpc_ui/#implement-a-backend)
for an adapter that delegates drawing and event handling to your toolkit.

Use `connect()` for an existing transport, or `UiHandler` to add UI processing
to an existing plugin connection. Your RPC handler can call Neovim while
handling requests on the same connection.

## Documentation

The [crate documentation](https://docs.rs/nvim-rpc-ui) covers backend
implementation, connection setup, grids and widgets, incremental painting,
keyboard input, and errors and shutdown. Build it locally with
`cargo doc -p nvim-rpc-ui --open`.

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development, validation, and
redraw performance measurements.
