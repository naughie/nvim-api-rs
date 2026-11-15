# nvim-rpc-sys

[![crates.io](https://img.shields.io/crates/v/nvim-rpc-sys.svg)](https://crates.io/crates/nvim-rpc-sys)
[![docs.rs](https://docs.rs/nvim-rpc-sys/badge.svg)](https://docs.rs/nvim-rpc-sys)

Generated descriptions of Neovim's API: function names and signatures, UI event
arguments, handle extension IDs, UI options, and API version constants. The input
is Neovim's `--api-info` output.

This crate provides metadata for typed bindings. Use [`nvim-rpc`](../nvim-rpc)
to send requests and decode results; it reexports RPC function descriptors
through `nvim_rpc::api`.

## Inspect a signature

Descriptors expose function names and type markers without making an RPC call:

```rust
use nvim_rpc_sys::{functions::NvimGetCurrentLine, types::{self, ApiFn}};

assert_eq!(NvimGetCurrentLine::NAME, "nvim_get_current_line");

fn string_result<F: ApiFn<Params = (), Return = types::String>>() {}
string_result::<NvimGetCurrentLine>();
```

See the [API documentation](https://docs.rs/nvim-rpc-sys) for handle methods,
UI metadata, type markers, and generated version information.

Building uses checked-in source and does not require Neovim. For regeneration
and contributor checks, see [codegen](../../codegen/README.md) and
[CONTRIBUTING.md](../../CONTRIBUTING.md).
