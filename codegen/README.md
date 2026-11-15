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

The command regenerates the checked-in bindings for `nvim-rpc-sys` and
`nvim-rpc`. Review the generated source and version changes together; see
[Generated bindings](../CONTRIBUTING.md#generated-bindings) for output details
and the contributor workflow.

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
