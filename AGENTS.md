# Project instructions

Apply these instructions alongside `~/.codex/AGENTS.md`. They specialize those conventions for this workspace.
Read the affected crate's README, source, manifests, and tests before changing
its design. [CONTRIBUTING.md](CONTRIBUTING.md) owns contributor procedures;
[codegen/README.md](codegen/README.md) owns generation instructions.

## Design priorities and crate boundaries

Semantic beauty is the primary design criterion: types, names, traits, and
operations must mean what they claim. Compiling successfully is insufficient.
Do not force a solution through a misleading trait implementation, wrapper,
bound, or change of abstraction. Reject such approaches. If no semantically
sound solution fits the requested contracts, explain the design conflict
rather than silently changing those contracts or producing the workaround.

Use `rmp-bytes` as the local Rust style baseline, applying the inherited rules
to the responsibility of each crate:

| Component | Responsibility |
| --- | --- |
| `crates/rmp-bytes` | MessagePack representations, incremental decoding, encoding states, and caller-selected I/O adapters |
| `crates/nvim-rpc-sys` | Generated Neovim signatures and metadata; this is not an FFI or transport layer |
| `crates/nvim-rpc` | Typed API calls, argument/result conversion, handles, RPC I/O, and UI event decoding |
| `crates/nvim-rpc-plugin` | Tokio tasks, shared message storage, handler dispatch, and response matching |
| `codegen` | Generates the checked-in bindings from Neovim metadata |

- Keep core operations usable without required allocation. Preserve the
  caller's choice of borrowed slices, owned values, exact-size iterators, and
  borrowed or lazy results; do not normalize them into `String`, `Vec`, or
  owned value trees merely for implementation convenience.
- Keep runtime choice and task scheduling outside the core call path. Optional
  Tokio/futures adapters provide I/O capabilities; enabling them must not start
  tasks. Allocation, buffering, channels, and spawning are intentional in the
  plugin layer when they serve its ownership and concurrency requirements.
- Keep `nvim-rpc` and `nvim-rpc-sys` dependencies under their own names in
  `workspace.dependencies`; `rmp-bytes` uses the `mp` alias. Inherit the workspace
  dependency definitions rather than duplicating paths and versions.

## Type and API contracts

- `Primitive` is sealed and belongs to the codec's built-in representations.
  Neovim handles and application records are not codec primitives. Preserve
  the distinction between the sealed type-marker vocabulary and the open
  conversion traits: `Like`, `Params`, `Decode`, `FromParams`, and `IntoResponse`.
- Keep conversion separate from I/O. A `to_primitive()` operation must remain
  a representation conversion rather than becoming a writer operation. Do not
  expose concrete implementation types unnecessarily when an opaque return
  type expresses the existing contract.
- Do not add a blanket `Primitive` implementation for `Option<T>` or silently
  choose its absence policy. `()` encodes an empty array; `Nil` encodes nil.
  Preserve MessagePack's integer ranges, Neovim's signed Integer marker, and
  unsigned 32-bit RPC IDs as distinct contracts.
- Preserve descriptor-based calls such as
  `nvim.request::<api::NvimSetCurrentLine>(("hello",))`. The core sender returns
  `Pending<F>` after writing; the plugin client's `request::<F, T>()` waits for
  and decodes the reply. Do not confuse the two in APIs or examples.
- Keep `Handler<W>` generic over the writer. Its associated argument and
  response types use `FromParams` and `IntoResponse`, which require
  `Send + 'static`; avoid repeating those bounds on the associated types.
  Plugin decode aliases retain shared `Bytes`. Keep the plugin facade's
  selective item reexports rather than reexporting the entire `nvim_rpc` crate.
- Do not use `Pin<Box<dyn Future>>`. Use RPITIT declarations with ordinary
  `async fn` implementations for async I/O traits rather than named future
  associated types. Reserve manual polling for adapters and state machines
  that need it.
- State required `Send` bounds explicitly, including encoding states and
  chunks where necessary. Do not use redundant nested `impl Trait` bounds to
  obtain an implicit auto-trait guarantee. Preserve local, non-`Send` codec
  and I/O use while retaining the existing typed RPC send contracts.
- Prefix private, unexported macros with `_`, such as `_impl_tuple`.

## RPC and plugin behavior

Preserve receive progress while handlers await nested Neovim calls. Register
response waiters before sending, flush plugin messages before waiting for
replies, and release the writer lock before awaiting a response. Replies to
incoming requests must follow Neovim's reverse request order, even when handler
completion order differs.

Keep cancellation and connection failure contracts intact: an interrupted
partial write cannot leave a reusable connection; dropping a response waiter
must not make its late reply invalidate the connection. Shutdown must wake
pending requests and stop the connection's handler tasks. For stdio plugins,
stdout carries RPC bytes and diagnostics go to stderr.

## Generated bindings

- Change generated APIs and rustdoc in `codegen/src/gen_api.rs` or
  `codegen/src/gen_api_sys.rs`, then regenerate both outputs. Do not make a
  generated-file-only fix. Review the generated source and manifest changes
  together using the procedure in the [codegen guide](codegen/README.md).
- Preserve diagnostics for unexpected metadata, including method-prefix
  mismatches; do not replace a reported mismatch with a silent skip. Preserve
  deliberate filtering of Lua-only signatures from the RPC facade and the
  distinction between metadata descriptors and RPC-callable functions.
- Ordinary generation aligns only `nvim-rpc-sys`'s package version and its
  workspace dependency requirement with the selected Neovim version. API
  levels are not release numbers: deprecation attributes use
  `#[deprecated(note = "Deprecated since Neovim API level N")]`, without `since`.
  Release automation has its own versioning procedure in `CONTRIBUTING.md`.

## Documentation and validation

Apply the inherited documentation rules to actual plugin and API workflows.
Use small, concrete examples that demonstrate the selected representations
and ownership. Keep the root README as the entry point, crate READMEs as usage
guides, rustdoc as API reference, and contributor procedures in
`CONTRIBUTING.md`; extend those homes rather than accumulating duplicate notes.

Function and method rustdoc links include parentheses, such as
``[`decode()`]``. Use third-person singular verbs when the documented item is
the omitted subject ("Writes…"); retain imperative wording for directions to
the reader ("Use…"). Preserve the nvim-rs acknowledgement in the plugin crate.

Run the applicable [validation checks](CONTRIBUTING.md#validation). Check core
crates in isolation when changing features, since workspace feature unification
can hide a dependency mistake. Preserve allocation, protocol, and lifecycle
coverage when touching those contracts. Mark prohibitively expensive boundary
tests `#[ignore = "..."]` rather than adding a feature solely to opt into them.
Live Neovim tests are also opt-in. Follow the contributor guide when selecting
and running them, and report what was verified.
