# ssmarshal

[![Crates.io](https://img.shields.io/crates/v/ssmarshal.svg?style=flat-square)](https://crates.io/crates/ssmarshal)

[Documentation](https://docs.rs/ssmarshal)

ssmarshal ("stupid simple marshaling") is a serde-based de/serialization
library. It is somewhat like [bincode](https://github.com/TyOverby/bincode),
but doesn't support String/Vec - this library is entirely zero-allocation, for
use in limited `no_std` contexts. The format is not incredibly compact, but
doesn't add extra fluff, and is quick to en/decode. The size of encoded values
will generally be the same as the in-memory representation, minus any padding.
For de/serializing a single value, a buffer of `size_of::<T>()` is always
enough.

All numbers are encoded as little-endian.

This format is not self-describing. To successfully deserialize a value, the
exact layout must be known ahead-of-time.

This is designed for doing IPC, not saving to disk or transferring over the
network. It may be useful for those cases, though.

This library is regularly fuzz tested with AFL for correct handling of
arbitrary input.
