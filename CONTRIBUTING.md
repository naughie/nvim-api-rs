# Contributing

The workspace separates MessagePack I/O, generated Neovim signatures, typed
calls, and Tokio plugin dispatch. Keep changes within the layer that owns the
behavior.

## Design constraints

The core libraries let callers choose borrowed or owned storage. Preserve paths
that encode slices and exact-size iterators directly and decode results without
building an owned value tree. Add allocation requirements only where ownership
requires them, and keep those boundaries explicit.

`rmp-bytes` is the coding style baseline. Its core codec and `nvim-api` leave
transport, receive storage, execution, and dispatch to the caller. Keep runtime
adapters optional. `nvim-api-plugin` owns the Tokio-specific tasks, shared frame
storage, and response matching.

Applications select argument representations through `Like` and result
representations through `Decode`. Preserve that extensibility when changing
generated calls or collection support. Use concrete futures; do not introduce
`Pin<Box<dyn Future>>` or task spawning into the core call path.

## Validation

Run these commands from the workspace root:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```

Workspace feature unification can enable adapters through other crates. Check
the core crates separately when changing feature gates or dependencies:

```sh
cargo test -p rmp-bytes --no-default-features
cargo test -p nvim-api --no-default-features
```

The allocation and efficiency tests check the borrowing and storage contracts.
Keep them alongside the crate whose behavior they verify.

### Neovim integration tests

The live tests are ignored by default. Set `NVIM_BIN` and select them explicitly:

```sh
NVIM_BIN=/path/to/nvim cargo test -p nvim-api --test live -- --ignored --test-threads=1
NVIM_BIN=/path/to/nvim cargo test -p nvim-api-plugin --test live -- --ignored
```

For an AppImage without FUSE, also set `APPIMAGE_EXTRACT_AND_RUN=1`.

### Large boundary tests

Some `rmp-bytes` tests traverse `u32::MAX`-length iterators. They are ignored in
routine runs because they process billions of elements. Run them when changing
length truncation or encoding boundaries:

```sh
cargo test -p rmp-bytes -- --ignored
```

## Generated bindings

Edit `codegen/src/gen_api.rs` or `codegen/src/gen_api_sys.rs` for changes to
generated code or its documentation, then regenerate both outputs:

```sh
cargo gen --nvim /path/to/nvim
```

The generator also aligns the `nvim-api-sys` version and workspace dependency
with the selected Neovim release. Review those manifest changes along with the
generated files. See [codegen/README.md](codegen/README.md) for output paths,
release downloads, and caching.

## Documentation

Keep the root README focused on choosing a crate and running an example. Crate
READMEs provide usage guides; rustdoc explains each public API's behavior,
ownership, errors, and cancellation contracts. Put contributor procedures here
and generator-specific details in the codegen README. Describe our encoding and
decoding choices where they affect callers, and link to external references
when broader protocol or dependency background is needed.
