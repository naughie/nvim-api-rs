# rmp-bytes

MessagePack encoding and decoding with borrowed slices, owned buffers, and
caller-selected I/O. Encoders produce byte chunks; decoders retain the input
buffer and visit array or map elements as you iterate.

## Encode a value

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

`Vec<u8>` is a writer, but allocation is not required. `StdWrite` also accepts
a `std::io::Cursor<&mut [u8]>` or another `std::io::Write` destination. Use
`ArrayIter` and `MapIter` for exact-size iterators, including mapped iterators.

Strings encode as MessagePack strings. Byte slices and `Vec<u8>` encode as
binary values. `Nil` encodes nil; `()` encodes an empty array. `Primitive` is
sealed, so application types convert to these representations. The crate does
not choose an encoding for `Option<T>`.

## Read without collecting

```rust
use rmp_bytes::decode::{Object, ObjectParser};

let bytes = b"\x92\xa3red\xa4blue";
let Object::Array(mut colors) = ObjectParser::new(bytes.as_slice())
    .next_object()
    .unwrap()
else {
    panic!("expected an array");
};

let Some(Object::Str(red)) = colors.next() else {
    panic!("expected a string");
};
assert_eq!(red.as_str().unwrap(), "red");
assert_eq!(colors.remaining_len(), 1);
```

With a slice input, string and binary payloads borrow that slice. Arrays and
maps are iterators, so you can inspect selected fields without allocating a
complete value tree. Numeric `to_*` accessors match exact variants; they do not
cast. For example, a positive fixint is decoded as `Object::U8`.

`ObjectParser` expects complete values. For fragmented input, use `SeekState`
to locate the next complete frame while retaining its bytes yourself.

## Optional adapters

There are no default features.

| Feature | Adds |
| --- | --- |
| `bytes` | `bytes::Bytes` decoding and `BufWriter` for `bytes::BufMut` |
| `tokio-write` | `TokioWrite` for Tokio `AsyncWrite` |
| `futures-write` | `FuturesWrite` for futures `AsyncWrite` |
| `rpc` | MessagePack-RPC request, response, and notification envelopes |

Async writes use `write_primitive(value).consume_async().await`. The adapters
do not choose an executor, spawn tasks, or flush the transport automatically. Use
`WriterSync::flush` or `WriterAsync::flush` at the boundary required by your
protocol. RPC support handles envelopes; the application remains responsible
for request IDs and dispatch.

See the crate documentation for fixed-buffer writes, custom async writers,
incremental framing, and RPC round trips. Contributor checks, including the
large boundary tests, are described in
[CONTRIBUTING.md](../../CONTRIBUTING.md).
