# rmp-bytes

[![crates.io](https://img.shields.io/crates/v/rmp-bytes.svg)](https://crates.io/crates/rmp-bytes)
[![docs.rs](https://docs.rs/rmp-bytes/badge.svg)](https://docs.rs/rmp-bytes)

MessagePack encoding and decoding with borrowed slices, owned buffers, and
caller-selected I/O. Encoders produce byte chunks; decoders retain the input
buffer and visit array or map elements as you iterate.

## Getting started

Tuples encode as arrays. `Map` and `Array` adapt collections without first
building another collection:

```rust
use rmp_bytes::encode::primitive::{Array, Map};
use rmp_bytes::encode::WriterSyncExt as _;

let words = ["red", "blue"];
let mut bytes = Vec::new();
bytes
    .write_primitive(Map([("colors", Array(&words[..]))]))
    .consume();
assert_eq!(bytes, b"\x81\xa6colors\x92\xa3red\xa4blue");
```

## Features

There are no default features.

| Feature | Adds |
| --- | --- |
| `bytes` | `bytes::Bytes` decoding and `BufWriter` for `bytes::BufMut` |
| `tokio-write` | `TokioWrite` for Tokio `AsyncWrite` |
| `futures-write` | `FuturesWrite` for futures `AsyncWrite` |
| `rpc` | MessagePack-RPC request, response, and notification envelopes |

See the [API documentation](https://docs.rs/rmp-bytes/latest/rmp_bytes/) for
decoding, representations, fixed-buffer writes, async adapters, incremental
framing, and RPC envelopes. Development and validation procedures are in
[CONTRIBUTING.md](../../CONTRIBUTING.md).
