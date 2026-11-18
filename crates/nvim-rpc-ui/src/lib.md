# Neovim UI integration

Implement [`Backend`] for your toolkit and present [`Frame`] values on the
thread that owns its widgets. Each frame contains complete screen [`State`],
changed regions in [`Damage`], and ordered [`Event`] values such as bells,
window-title changes, and API errors.

A frame owns the data it retains. It remains valid after later redraws and
connection shutdown, and can cross threads independently of the backend's
widget objects. A backend may retain a frame and schedule painting through its
toolkit's event loop. Handle its events in order even when painting is deferred.

## Implement a backend

Implement [`Backend::present()`] in your application or toolkit adapter. The
trait has no `Send`, `Sync`, or `'static` bound, so it can contain widget objects
that must stay on their UI thread. This adapter handles each event and redraws
the complete state using application-provided callbacks:

```rust
use nvim_rpc_ui::{Backend, Event, Frame, State};
use std::convert::Infallible;

struct Renderer<D, E> {
    draw: D,
    handle_event: E,
}

impl<D, E> Backend for Renderer<D, E>
where
    D: FnMut(&State),
    E: FnMut(Event),
{
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        for event in frame.events {
            (self.handle_event)(event);
        }
        (self.draw)(&frame.state);
        Ok(())
    }
}

let mut drawn = 0;
let mut handled = 0;
{
    let mut renderer = Renderer {
        draw: |_: &State| drawn += 1,
        handle_event: |_: Event| handled += 1,
    };
    renderer.present(Frame {
        sequence: 1,
        state: Default::default(),
        damage: Default::default(),
        events: vec![Event::Bell],
    })?;
}
assert_eq!((drawn, handled), (1, 1));
# Ok::<(), Infallible>(())
```

A backend may schedule painting through the toolkit's event loop. Process
`Frame::events` in receive order, including bells, title changes, and connection
requests, even when an intermediate state will not be painted.

## Start an embedded editor

Use [`Embedded::spawn()`] within a running Tokio runtime with I/O enabled.
Configure a [`tokio::process::Command`] to select the executable, arguments,
environment, and working directory. Start Neovim without `--headless`; the
embedded editor attaches with the requested dimensions and UI options before
returning.

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

[`Embedded::present_next()`] waits for a frame and calls [`Backend::present()`]
on the thread polling the future. Drive it on the toolkit's UI thread. It
returns `true` after presenting a frame and `false` after the frame stream
ends. A backend error consumes that frame and leaves the connection available
for recovery or shutdown. Cancelling the wait before a frame arrives does not
consume a frame.

Use [`Embedded::spawn_with_handler()`] to handle application RPC methods on the
same connection. Requests and non-UI notifications reach that handler;
`redraw` and `nvim_error_event` update the UI. Handler requests may call Neovim
while waiting for a nested API response.

## Connect an existing transport

[`connect()`] accepts read and write halves, an application RPC handler, and
[`ConnectionOptions`]. It returns a [`Connection`] with a UI [`Client`] and a
[`Frames`] receiver. Attach using [`Client::attach()`] before waiting for frames.
Use [`Connection::present_next()`] to receive and present on the toolkit's UI
thread. The connection does not own an externally managed Neovim process.

The RPC connection is provided by `nvim-rpc-plugin`. Use [`Client::nvim()`] to
call its [`nvim_rpc_plugin::Neovim::request()`] and
[`nvim_rpc_plugin::Neovim::notify()`] methods. A request waits for and decodes its
response; a notification returns after writing. Multiple client handles can
use the same connection. UI action helpers such as resize and focus send
notifications, and Neovim reports execution errors through [`Event::RpcError`]
in a later frame.

If your runtime and toolkit have separate event loops, receive frames with
[`Frames::recv()`] and transfer them to the toolkit thread. Present the owned
frame through your backend there. Receiving does not wait for painting.
Unconsumed updates are combined into the latest state with full-repaint damage,
while retaining their events in order. Exceeding [`Limits::max_pending_events`]
ends UI processing and closes the shared connection.

## Add UI handling to a plugin

Wrap an existing [`nvim_rpc_plugin::Handler`] with [`UiHandler`] and pass it to
[`nvim_rpc_plugin::connect()`]. Its client performs ordinary API calls and UI
attachment on the same connection:

```rust,no_run
# async fn connect<R, W, H>(reader: R, writer: W, handler: H)
#     -> Result<(), nvim_rpc_plugin::Error>
# where R: tokio::io::AsyncRead + Unpin + Send + 'static,
# W: tokio::io::AsyncWrite + Unpin + Send + 'static,
# H: nvim_rpc_plugin::Handler<W> {
use nvim_rpc_ui::{Limits, UiHandler, UiOptions, api};

let (handler, frames) = UiHandler::new(handler, Limits::default());
let mut connection = nvim_rpc_plugin::connect(reader, writer, handler, Default::default())?;
let nvim = &connection.client;
let options = UiOptions { ext_linegrid: true, ..UiOptions::default() };
let _: nvim_rpc_plugin::Object = nvim
    .request::<api::NvimUiAttach, _>((100i64, 30i64, &options)).await?;
# connection.close_and_wait().await?;
# drop(frames);
# Ok(()) }
```

Use [`Frames::present_next()`] on your backend's UI thread or [`Frames::recv()`]
for application-managed delivery. UI processing retains at most one pending
frame. Coalescing preserves every event up to [`Limits::max_pending_events`];
unflushed events are bounded separately by the model. Application-held frames
are outside these limits.

A UI protocol, event-limit, or application notification-decoding error closes
the shared connection. The frame stream returns preceding valid updates, then
its terminal [`FrameError`], then ends. Use the plugin connection's
[`wait()`](nvim_rpc_plugin::Connection::wait()) method to observe transport
errors and request-handler failures.

## Draw grids and widgets

Attach with `UiOptions::ext_linegrid` enabled. Enable widget extensions such as
`ext_cmdline`, `ext_popupmenu`, or `ext_messages` only for components your backend
renders. Multigrid uses `ext_multigrid`. The older cell-at-a-time grid protocol
returns `ProtocolError::LegacyGrid`.

Read grid cells from `frame.state.grids`. Cell text is UTF-8; an empty string
marks the continuation cell of a wide character. Each row also records whether
it wraps onto the following row. Resolve each cell's highlight ID through
`state.highlights`. Absent highlight colors use `state.default_colors`, whose
unset colors use your toolkit's defaults. `ext_hlstate` adds semantic highlight
information to the concrete highlight attributes.

Grid contents and positions have separate lifetimes. A hidden grid retains its
cells. Use `state.placements` to place normal, floating, external, and message
grids; external grids need application-managed windows or panels. Look up a
window's buffer positions in `state.viewports` using the same grid ID.

Grid and floating-window coordinates count cells. Command-line cursor positions
and command-line popup anchors count UTF-8 bytes. Font metrics, text shaping,
clipping, IME preedit, and conversion to pixel coordinates belong to the backend.

### Incremental painting

`Frame::damage` records changes since the preceding published frame. Use its
grid rectangles to update affected cell regions, and its flags to update layout,
cursor, or widgets. With `damage.full`, redraw from the complete current state
and remove grids or placements that are no longer present.

To skip painting intermediate states, retain the last painted `Arc<State>` and
call `frame.state.diff_since(&last_painted)`. The returned damage describes
changes from that snapshot, including removed grids. It may conservatively
include unchanged regions. Update the baseline after painting succeeds.

This comparison covers state, not one-time events. Handle every frame's events
in receive order, or combine pending frames with `Frame::coalesce()`. Coalescing
retains the latest state and all events in order, requests a full repaint, and
returns an error if the supplied event limit would be exceeded. Leave the
existing pending frame available for handling when that happens.

Window exposure, font or DPI changes, and graphics-device loss can also require
repainting without receiving a new frame.

## Keyboard and text input

Translate platform keyboard events into logical characters and modifiers using
`KeyInput`. Use `KeyInput::Text` for IME commits and `KeyInput::Sequence` for
explicit Neovim key notation.

```rust
use nvim_rpc_ui::{KeyInput, Modifiers};

let key = KeyInput::from_ascii(b'a', Modifiers { control: true, ..Default::default() })
    .unwrap();
assert_eq!(key.to_nvim().as_str(), "<C-a>");
let text = KeyInput::Text { text: "日本語<Esc>" };
assert_eq!(text.to_nvim().as_str(), "日本語<LT>Esc>");
let sequence = KeyInput::Sequence { keys: "ihello<Esc>" };
assert_eq!(sequence.to_nvim().as_str(), "ihello<Esc>");
```

`from_char()` accepts Unicode scalars. `from_ascii()` accepts ASCII bytes,
including control characters; a byte alone cannot distinguish Backspace from
Ctrl-H or Tab from Ctrl-I. Construct `KeyInput::SpecialKey` directly when the
platform provides the corresponding key information. `Modifiers` records Shift,
Control, Alt/Meta, and Command/Super. Do not apply Shift again to a character
already translated by the keyboard layout.

`KeyInput<'a>` is `Copy` and borrows text as `&'a str`. `to_nvim()` takes the input
by value and returns the encoded notation. Its result may outlive the input
wrapper while the borrowed source text remains valid.

Send keys with [`Client::send_key()`]; keep borrowed text alive until the
future completes or is dropped. [`Client::input()`] accepts Neovim notation,
so escape literal `<` as `<LT>` when using it directly. Both methods retry
partial input acceptance and serialize concurrent input sequences. Input
already accepted by Neovim remains accepted if the call is cancelled. Use
[`Client::paste()`] for clipboard text blocks; it returns whether Neovim
accepted the paste.

## Use your own RPC dispatcher

Create `Ui::new(backend)` and pass decoded notifications by reference to
`Ui::handle_notification()` in wire order on the backend's UI thread. It
handles `redraw` and `nvim_error_event`, and returns `false` for other methods
without consuming their arguments. `Ui::redraw()` accepts an already-routed
redraw notification by value. Notifications without `flush` update retained
state without calling the backend. A redraw notification containing several
flushes presents each completed frame in order. Use `Ui::with_limits()` to
configure state limits.

Your dispatcher remains responsible for RPC requests, responses, and other
notifications. Keep receiving messages while a request handler awaits a nested
Neovim call. The input bytes are borrowed during `redraw()`; frames own the data
they retain. Access toolkit-specific backend operations through `Ui::backend()`
or `Ui::backend_mut()`.

For applications that need to publish frames separately from presentation,
[`Model::apply_notification()`] provides state updates and a callback for each
frame. Use `Array::with_borrowed_buffer()` to borrow the parameter bytes while
the original array keeps its position for further dispatch. API errors publish the last
flushed state with any earlier pending events before the error event; unfinished
screen changes remain pending until the next flush.
Unknown redraw events are exposed as `Event::Unknown`; known events accept
additional trailing arguments. `Value::String` preserves API string bytes,
including non-UTF-8 data, in options and event metadata. Grid cells and other
rendered UI text use UTF-8 strings.

## Errors and shutdown

UI notification and resource-limit failures are reported as [`FrameError`].
With a supplied [`Connection`], [`Connection::present_next()`] also reports
connection failures after pending frames have been consumed. The underlying
RPC error remains available through [`Connection::wait()`].

[`PresentationError::Backend`] means the backend rejected a frame. The frame
has been consumed and is not queued again. Account for its events before
resuming, and compute damage from the last successfully painted state with
[`State::diff_since()`] or redraw the complete current state. Cancelling
`present_next()` while it waits for a frame leaves that frame available.

For an application-managed dispatcher, [`Ui::redraw()`] returns
[`RedrawError::Protocol`] for notification errors and [`RedrawError::Backend`]
for presentation errors. Either failure stops the current notification
immediately. [`Ui::is_failed()`] then returns `true` and rejects subsequent
redraws. Create a fresh UI attachment before continuing; the unprocessed
notification may contain required updates. Earlier frames remain valid.

Configure [`Limits`] to bound grids, grid cells, individual strings and binary
values, collections, retained messages, value nesting, and pending events.
These limits do not bound the total memory retained by application-held frames.
[`ConnectionOptions::rpc`] configures the underlying RPC connection.

Dropping a request future while it awaits a response leaves the editor
operation running; its eventual response is consumed. Cancelling a partial RPC
write closes the connection. EOF wakes outstanding requests and ends frame
reception after pending updates have been delivered.

[`Connection::detach_ui()`] removes the UI attachment while leaving RPC active.
[`Connection::stop()`] requests connection shutdown; [`Connection::wait()`]
waits for cleanup, and [`Connection::close_and_wait()`] combines them. Cancelling
a wait leaves it available for another attempt. Drain frames before waiting
for natural completion, or call `stop()` first. Finish or drop outstanding
write futures so their transport resources can be released.

[`Embedded::shutdown()`] terminates and reaps the child without saving modified
buffers. Dropping `Embedded` terminates it; explicit shutdown reports process
errors. [`Embedded::wait_connection()`] waits for its RPC connection, while
[`Embedded::wait()`] waits for the child process. A supplied `Connection` never
terminates an externally owned process.
