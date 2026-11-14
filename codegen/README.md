# codegen

Generates the Neovim API descriptors and handle call builders from
`nvim --api-info`. Run it from the workspace root using the `cargo gen` alias.

## Use a local Neovim

```sh
cargo gen --nvim /path/to/nvim
```

When running an AppImage without FUSE, pass its extraction setting through to
the child process:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 cargo gen --nvim /path/to/nvim.appimage
```

The command prints the Neovim version, then writes and formats:

- [`crates/nvim-rpc-sys/src/generated.rs`](../crates/nvim-rpc-sys/src/generated.rs):
  function and event descriptors, UI options, extension IDs, and version constants.
- [`crates/nvim-rpc/src/generated.rs`](../crates/nvim-rpc/src/generated.rs):
  the `api` descriptor re-exports, handle methods, and UI option encoding.

It also sets `nvim-rpc-sys`'s package version to Neovim's `major.minor.patch`
and updates the workspace dependency requirement accordingly. Other package
versions are unchanged. Review both generated files and the manifest changes
together.

## Download a release

Without `--nvim`, the tool downloads a Neovim AppImage. This mode supports Linux
x86_64 and aarch64:

```sh
cargo gen 0.12.6
```

Omit the version to fetch the `stable` release:

```sh
cargo gen
```

Downloaded executables are cached under `tmp/nvim/`. The no-version form downloads
the stable release again on each run; specifying a version allows the matching
cached executable to be reused.

## Change the generator

Edit `src/gen_api_sys.rs` for metadata descriptors or `src/gen_api.rs` for RPC
descriptor reexports, handle call builders, and UI option encoding. Change
generated documentation in these generators as well, then regenerate both
outputs.

The metadata output includes Lua-only signatures. The RPC output omits functions
with a `LuaRef` in their parameter or return types. Deprecated functions and
handle methods receive `#[deprecated]` with their API level. A method name that
does not match its handle prefix produces a warning and remains available only
as a function descriptor.

Check the generated code and examples afterward:

```sh
cargo test -p codegen
cargo test --workspace --features nvim-rpc/bytes,nvim-rpc/tokio,nvim-rpc/futures
```
