# nvim-api-sys

Generated descriptions of Neovim's API: function names and signatures, UI event
arguments, handle extension IDs, UI options, and API version constants. The input
is Neovim's `--api-info` output.

This crate provides metadata for typed bindings. Use [`nvim-api`](../nvim-api)
to send requests and decode results; it reexports RPC function descriptors
through `nvim_api::api`.

## Function signatures

Each function descriptor implements `ApiFn`. Its associated types are Neovim type
markers, leaving the Rust argument and result representations to the binding:

```rust
use nvim_api_sys::{functions::NvimGetCurrentLine, types::{self, ApiFn}};

assert_eq!(NvimGetCurrentLine::NAME, "nvim_get_current_line");

fn string_result<F: ApiFn<Params = (), Return = types::String>>() {}
string_result::<NvimGetCurrentLine>();
```

`types::String` describes a String in the API. It does not hold text or require
`std::string::String`. Likewise, `ArrayOf<types::String>` describes the elements
without choosing a `Vec`, slice, or iterator.

Buffer, window, and tabpage methods implement both `ApiFn` and `ApiMethod`:

```rust
use nvim_api_sys::functions::NvimBufGetLines;
use nvim_api_sys::types::{ApiFn, ApiMethod};

assert_eq!(<NvimBufGetLines as ApiFn>::NAME, "nvim_buf_get_lines");
assert_eq!(<NvimBufGetLines as ApiMethod>::NAME, "get_lines");
```

The full function signature includes the receiver as its first parameter.
`ApiMethod::On` identifies that receiver, while `ApiMethod::Params` contains the
remaining parameters. Generated struct fields describe parameter names and
markers; constructing one does not perform an RPC call.

## UI metadata

`ui_events` contains event descriptors implementing `types::UiEvent`. For example,
`GridResize::Params` is `(types::Int, types::Int, types::Int)` for the grid ID,
width, and height.

`UiOptions` holds option values rather than type markers:

```rust
use nvim_api_sys::UiOptions;

let options = UiOptions {
    ext_linegrid: true,
    ..UiOptions::default()
};
assert!(options.rgb);
assert!(!options.ext_multigrid);
```

## Generated version

This crate's package version matches the Neovim release used for generation.
Deprecated API descriptors carry a `#[deprecated]` attribute whose note gives
the API level at which they were deprecated.

`VERSION`, `API_LEVEL`, `API_COMPATIBLE`, and `API_PRERELEASE` describe the Neovim
metadata used to generate the source. They do not describe a running server;
query that server's API information when checking compatibility.

`ext_types::{Buffer, Window, Tabpage}::ID` gives each handle's MessagePack extension
tag. These types are markers; runtime handle values live in `nvim-api`.

The generated source is checked in. Building this crate does not run Neovim or
the generator. See [codegen](../../codegen/README.md) for regeneration instructions.
