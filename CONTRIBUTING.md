# Contributing

The workspace separates MessagePack I/O, generated Neovim signatures, typed
calls, and Tokio plugin dispatch. Keep changes within the layer that owns the
behavior.

`nvim-rpc-ui` owns retained redraw state, frame delivery, UI input helpers, and
embedded-process lifetimes. It uses `nvim-rpc-plugin` for RPC dispatch and the
client. Keep request matching, writer serialization, and connection shutdown in
the plugin layer. Applications implement `Backend` and own toolkit objects and
integration.

## Design constraints

The core libraries let callers choose borrowed or owned storage. Preserve paths
that encode slices and exact-size iterators directly and decode results without
building an owned value tree. Add allocation requirements only where ownership
requires them, and keep those boundaries explicit.

`rmp-bytes` is the coding style baseline. Its core codec and `nvim-rpc` leave
transport, receive storage, execution, and dispatch to the caller. Keep runtime
adapters optional. `nvim-rpc-plugin` owns the Tokio-specific tasks, shared frame
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
cargo test -p nvim-rpc --no-default-features
cargo test -p nvim-rpc-ui
```

The allocation and efficiency tests check the borrowing and storage contracts.
Keep them alongside the crate whose behavior they verify.

### Neovim integration tests

The live tests are ignored by default. Set `NVIM_BIN` and select them explicitly:

```sh
NVIM_BIN=/path/to/nvim cargo test -p nvim-rpc --test live -- --ignored --test-threads=1
NVIM_BIN=/path/to/nvim cargo test -p nvim-rpc-plugin --test live -- --ignored
NVIM_BIN=/path/to/nvim cargo test -p nvim-rpc-ui --test session -- --ignored --test-threads=1
```

For an AppImage without FUSE, also set `APPIMAGE_EXTRACT_AND_RUN=1`.

Backend presentation, UI model replay, allocation/sharing, mock connection,
and input tests run without Neovim. Preserve every flush in local `Ui` dispatch
and ordered events when connection frames coalesce. Check that a failed
`Ui::redraw()` stops processing and that cancelling a frame wait does not consume
a frame. Backend failures during connection presentation must leave the
connection available for recovery or shutdown.

UI connections share the plugin dispatcher and response matching. Their tests
check ordered events under coalescing, nested requests while presentation is
idle, concrete error delivery, and shutdown of later callbacks. Run shared RPC
lifecycle tests in `nvim-rpc-plugin`. Connection waits must remain reusable
after cancellation, including while a writer guard is still held. Supplied
transports must not acquire process ownership.

### UI redraw performance

Measure decoding, redraw processing, frame publication, and synthetic CPU
painting in a release build:

```sh
cargo run --release -p nvim-rpc-ui --example replay -- 100000
```

The immediate-delivery workloads retain the previous frame and replay typing,
whole-row scrolling, and 120-entry rows on a 120×40 grid. Rows cover ASCII,
repeated CJK, combining sequences, emoji, and multi-character strings. CJK
working sets of 256, 1,024, and 4,096 scalars show how reuse varies with
vocabulary size. Two further streams draw from 15,360 CJK scalars to exercise
cache churn; the mostly unique stream replaces every eighth entry with one
recurring character. Each row entry is encoded separately, without a repeat
run. Fixtures are encoded before measurement.

These workloads measure decoding, model updates, and immediate frame delivery
without painting. Throughput follows 1,000 warm-up frames. A separate pass of
up to 1,000 frames counts allocation and reallocation calls on the current
thread; setup and fixture encoding are excluded.

The burst workloads use two 240×80 grids and paint once per eight flushes. They
compare `Frame::coalesce()` with `Frame::coalesce_incremental()` for cursor-only
updates and sparse cell updates. A CPU backend traverses every cell selected by
the resulting damage and reads its text and highlight. The output reports total
time, CPU paint time, cells visited, and cursor updates. Full repaint visits
38,400 cells per paint; incremental cursor updates visit none, and the sparse
updates visit two three-cell bounding regions. The frame count is rounded up
to a multiple of eight for these workloads.

These measurements exclude Neovim RPC latency, real text shaping, rasterization,
FFI transfer, toolkit work, and GPU presentation. Burst timings reflect work
selected by damage with a synthetic CPU cost. Measure a real backend separately
for GUI frame rates. Record the workload and build configuration when comparing
measurements.

The UI allocation and sharing tests run with the regular test suite. When
changing frame storage or grid updates, check that retained frames stay stable
and that unchanged grids and rows remain shared.

#### Cell text reuse

Grid-line decoding uses a private per-model table of 512 slots for strings up to
64 UTF-8 bytes. A borrowed string hashes to one slot; an exact match clones its
`Arc<str>`, and a miss replaces that slot. Collisions affect reuse only. Cells
and retained frames own their references, so replacement cannot invalidate them.
The table is allocated lazily and retains at most 32 KiB of string payload, plus
slot storage and `Arc` headers. Empty strings and single ASCII characters use
the existing shared fast path. Longer strings use ordinary allocation. All
strings still obey the model's string limit. These cache bounds are internal
choices, not new protocol limits; text bytes are preserved without normalization.

Reuse is sampled over 128 lookups. If fewer than 96 hit, the next 2,048 eligible
entries bypass lookup and insertion before sampling resumes. Existing slots stay
available for later probes. Empty strings, single ASCII characters, and strings
longer than 64 bytes do not advance this sampling state. This policy favors
strong reuse while limiting hashing and reference-counting work on unique text.

The following representative comparison used the same instrumented replay
example on baseline `8d0f27c` and the cache implementation: Linux x86_64,
Intel Core i7-14700KF, rustc 1.99.1, release build, 100,000 measured frames per
workload, pinned to CPU 0 with `taskset -c 0 target/release/examples/replay 100000`.
The allocation column compares baseline and cache calls per frame. Repeat the
cache comparison with the replay example from `0cfe6df` on both revisions;
that source does not require the incremental-coalescing API. Timings vary with
the machine and workload.

| Workload | Baseline µs/frame | Cache µs/frame | Allocations/frame, baseline → cache |
| --- | ---: | ---: | ---: |
| ASCII row | 13.86 | 13.94 | 8 → 8 |
| Repeated CJK | 15.96 | 15.39 | 128 → 8 |
| Combining sequences | 16.26 | 15.57 | 128 → 8 |
| Emoji | 16.50 | 15.72 | 128 → 8 |
| Multi-character strings | 16.66 | 15.81 | 128 → 8 |
| 256-scalar working set | 16.09 | 16.28 | 128 → 121.55 |
| 1,024-scalar working set | 16.18 | 16.22 | 128 → 122.91 |
| 4,096-scalar working set | 16.15 | 16.18 | 128 → 127.89 |
| Mostly unique CJK | 16.19 | 16.19 | 128 → 127.13 |
| CJK cache churn | 16.47 | 16.23 | 128 → 128 |

Repeated Unicode rows removed 120 text allocations per frame with a small
throughput improvement. Low-reuse workloads stayed close to the baseline in
repeated pinned runs. The always-active table made mostly unique text and cache
churn about 13% slower; a `HashSet` plus FIFO prototype had approximately 30%
churn overhead. Sampling and bypassing avoids paying the lookup/replacement
cost on every low-reuse entry. A shorter 512-entry bypass left up to about 3%
overhead in the measured working sets; 2,048 entries reduced it further while
retaining probes for a change in vocabulary.

Capacity trials with bypass disabled used 256, 512, and 1,024 slots. At 256 slots,
collisions in the combining-sequence fixture left 68 allocations per frame
instead of 8. Increasing 512 to 1,024 slots reduced allocations for the 256-scalar
working set from 52.01 to 35.67, and the 1,024-scalar set from 108.56 to 85.34,
with similar throughput and twice the retention bound. The 512-slot choice keeps
a 32 KiB payload budget while covering the tested small repeated vocabularies;
it does not guarantee reuse for all text that would fit within that budget.

### RPC reply-stack performance

Measure incoming reply registration, completion, and retirement in a release
build:

```sh
cargo test -p nvim-rpc-plugin --release --lib benchmark_reply_stacks -- --ignored --nocapture
```

The benchmark covers fresh and reused bursts at depths from 1 to 16,384, plus
interleaved push/complete/pop operations above unfinished older requests. It
includes the reply-state mutex and readiness notifications, but excludes RPC
decoding, transport I/O, and handler tasks. Compare repeated runs with the same
workload and build configuration.

Reply registration scans shallow stacks and builds a standard `HashSet` when
adding a request above 64 pending slots. Release-mode comparisons against linear
scans and always-on indexing placed the sustained-depth crossover around 64
slots; constructing the index can still cost more for bursts just above that
depth. Membership includes completed replies until they are popped for writing.
The index remains active until the stack drains, then shallow scans resume while
the allocated set storage is retained for reuse.

### Large boundary tests

Some `rmp-bytes` tests traverse `u32::MAX`-length iterators. They are ignored in
routine runs because they process billions of elements. Run them when changing
length truncation or encoding boundaries:

```sh
cargo test -p rmp-bytes -- --ignored
```

## Generated bindings

Edit `codegen/src/gen_api_sys.rs` for metadata descriptors or
`codegen/src/gen_api.rs` for RPC descriptor reexports, handle call builders, and
UI option encoding. Change generated documentation in these generators as well,
then regenerate both outputs:

```sh
cargo gen --nvim /path/to/nvim
```

The generator writes and formats:

- [`crates/nvim-rpc-sys/src/generated.rs`](crates/nvim-rpc-sys/src/generated.rs):
  function and event descriptors, UI options, extension IDs, and version constants.
- [`crates/nvim-rpc/src/generated.rs`](crates/nvim-rpc/src/generated.rs):
  the `api` descriptor reexports, handle methods, and UI option encoding.

It also sets `nvim-rpc-sys`'s package version to Neovim's `major.minor.patch`
and updates its workspace dependency requirement. Other package versions are
unchanged. Review both generated files and the manifest changes together.
See [codegen/README.md](codegen/README.md) for local executable and release
download options, including caching.

The metadata output includes Lua-only signatures. The RPC output omits functions
with a `LuaRef` in their parameter or return types. Deprecated functions and
handle methods receive `#[deprecated]` with their API level. A method name that
does not match its handle prefix produces a warning and remains available only
as a function descriptor.

Check the generator and run the [workspace validation](#validation) after
regeneration:

```sh
cargo test -p codegen
```

### Scheduled Neovim updates

The [Update Neovim bindings workflow](.github/workflows/update-neovim.yml) checks
upstream releases daily at 02:17 UTC (11:17 Japan time). It selects the highest
numeric stable release, excluding drafts, prereleases, and the moving `stable`
and `nightly` tags. If `nvim-rpc-sys` already targets that version or a newer one,
the run finishes without changes.

For a newer release, the workflow runs `cargo gen` with the explicit version.
It keeps `nvim-rpc-sys` aligned with Neovim, increments the patch versions of all
other packages under `crates/`, and updates workspace dependency requirements
and `Cargo.lock`. The generator's ordinary local behavior is unchanged. Bumps
are calculated from a fresh checkout of the default branch, so retries do not
accumulate patch increments.

The workflow runs the validation commands above, including both live integration
test suites against the downloaded Neovim. Only after validation passes does it
create or update a PR from `automation/update-neovim`. Until merged, that PR is
reused for subsequent updates. Review the generated API and version choices before
merging, especially when Neovim's minor or major version changes. Merge the PR
manually to start [publishing to crates.io](#publishing-neovim-updates).

To enable it:

1. Commit the workflow and its `.github/scripts/` helpers to the repository's
   default branch. Scheduled workflows run only from that branch.
2. In **Settings → Actions → General**, enable GitHub Actions and allow
   `actions/checkout` and `peter-evans/create-pull-request` if your action policy
   restricts which actions may run.
3. Under **Workflow permissions**, enable **Allow GitHub Actions to create and
   approve pull requests** and save. The workflow declares `contents: write`
   and `pull-requests: write` itself; the default token permission can remain
   read-only. Organization policy must also permit these settings.
4. Open **Actions → Update Neovim bindings → Run workflow** for an initial check.
   It always checks out the repository's default branch, including manual runs.
   After that, the daily schedule runs automatically.

No custom secrets are required; release lookup and PR creation use the built-in
`GITHUB_TOKEN`. Validation runs inside the update workflow. Additional PR checks
may require approval when the PR is created with that token; see
[GitHub's token-trigger rules](https://docs.github.com/en/actions/concepts/security/github_token#when-github_token-triggers-workflow-runs)
if adding required checks.

Change the workflow's `schedule` entry to adjust its frequency. GitHub can delay
or drop scheduled runs and disables public-repository schedules after 60 days
without repository activity. Re-enable a disabled workflow from the Actions
page; see [GitHub's schedule documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule).

The release scripts require Python 3.11 or newer. Their focused tests run without
downloading Neovim, uploading crates, or changing the working tree:

```sh
python3 -B -m unittest discover -s .github/scripts -p 'test_*.py'
```

### Publishing Neovim updates

The [Publish Neovim update workflow](.github/workflows/publish-neovim.yml) runs
when a PR from this repository's `automation/update-neovim` branch is merged into
the default branch. Closing a PR without merging, merging a PR from another branch,
or merging a fork's PR does not publish crates. The workflow checks out the
PR's merge commit, checks formatting and runs the all-features workspace tests,
then uses crates.io trusted publishing to obtain a temporary upload token.

It publishes `rmp-bytes`, `nvim-rpc-sys`, `nvim-rpc`, `nvim-rpc-plugin`, and
`nvim-rpc-ui` in that order. Each `cargo publish` verifies the packaged crate and
waits for its registry index entry before the next dependent crate is published.
`codegen` is not published. The workflow does not require a long-lived crates.io
token in GitHub secrets.

Complete this setup before merging the first automated update PR:

1. Publish an initial version of each crate manually if it is not already on
   crates.io. Trusted publishing requires existing crates that you own. From a
   clean checkout with release-ready versions and a current `Cargo.lock`, log
   in using `cargo login`, then publish missing crates in dependency order:

   ```sh
   cargo publish -p rmp-bytes --locked --registry crates-io
   cargo publish -p nvim-rpc-sys --locked --registry crates-io
   cargo publish -p nvim-rpc --locked --registry crates-io
   cargo publish -p nvim-rpc-plugin --locked --registry crates-io
   cargo publish -p nvim-rpc-ui --locked --registry crates-io
   ```

2. In GitHub **Settings → Environments**, create an environment named
   **crates-io**. Under deployment branches and tags, allow only the default
   branch. Leave required reviewers and wait timers unset so publishing starts
   automatically after you merge. If the action policy is restricted, also
   allow `rust-lang/crates-io-auth-action` under **Settings → Actions → General**.
3. For **each of the five crates**, open crates.io **Settings → Trusted
   Publishing**, add a GitHub publisher, and use these values:

   | Field | Value |
   | --- | --- |
   | Repository owner | `naughie` |
   | Repository name | `nvim-rpc-rs` |
   | Workflow filename | `publish-neovim.yml` |
   | Environment | `crates-io` |

   The workflow field is the filename alone, without `.github/workflows/`.
   See the [crates.io trusted publishing guide](https://crates.io/docs/trusted-publishing)
   for registry prerequisites and configuration.
4. Commit both workflows and their helper scripts to the default branch. Once
   configured, review and merge an automated Neovim update PR normally, then
   follow **Actions → Publish Neovim update** to check the upload results.

For a failed or partial release, fix the external cause (for example, a missing
trusted publisher entry) and use **Re-run failed jobs** on that publishing run.
The same merged commit is used again. Already published versions are skipped
after checking that they are not yanked and are visible in the registry index;
the remaining versions are uploaded in dependency order. Registry errors stop
the run. If the source or versions need fixing, merge a corrective update PR
with fresh versions for crates that were already published.

## Documentation

Keep every README within its role as an entry point: purpose, prerequisites,
a small getting-started workflow, and links to further documentation. A README
does not replace a usage guide, API reference, or contributor guide. Put detailed
API behavior, representations, ownership, errors, and cancellation contracts in
rustdoc, and contributor procedures and generator internals here. Keep the
codegen README focused on running the tool. Describe encoding and decoding
choices in the relevant API documentation, and link to external references when
broader protocol or dependency background is needed.
