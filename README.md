[![Build Status](https://travis-ci.org/hjr3/hyper-timeout.svg?branch=master)](https://travis-ci.org/hjr3/hyper-timeout)
[![crates.io](https://img.shields.io/crates/v/hyper-timeout.svg)](https://crates.io/crates/hyper-timeout)

# hyper-timeout

A timeout aware connector to be used with hyper `Client`.


## Problem

At the time this crate was created, hyper does not support timeouts. There is a way to do general timeouts, but no easy way to get connect, read and write specific timeouts.

## Solution

There is a `TimeoutConnector` that implements the `hyper::Connect` trait. This connector wraps around `HttpConnector` or `HttpsConnector` values and provides timeouts.

**Note:** Because of the way `tokio_proto::ClientProto` works, a read or write timeout will return a _broken pipe_ error.

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
hyper-timeout = "0.1"
```

Next, add this to your crate:

```rust
extern crate hyper_timeout;
```

See the [client example](./examples/client.rs) for a working example.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
