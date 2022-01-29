# crates-io-api

[![Crate][cratesioimg]][cratesio]
[![API Docs][docsrsimg]][docsrs]

[cratesio]: https://crates.io/crates/crates_io_api
[cratesioimg]: https://img.shields.io/crates/v/inkpad-runtime.svg
[docsrs]: https://docs.rs/crates_io_api
[docsrsimg]: https://img.shields.io/badge/current-docs-brightgreen.svg
[crawlerpolicy]: https://crates.io/policies#crawlers
[reqwest]: https://github.com/seanmonstar/reqwest

A Rust API client for the [crates.io](https://crates.io) API.

This crate aims to provide an easy to use and complete client for retrieving
detailed information about Rusts crate ecosystem.

The library uses the [reqwest][reqwest] HTTP client and provides both an async
and synchronous interface.

Please consult the official [Crawler Policy][crawlerpolicy] before using this
library. 
A rate limiter is included and enabled by default.

## Usage

For usage information and examples, check out the [Documentation][docsrs].

### rustls

By default the system TLS implementation is used.

You can also use [rustls](https://github.com/rustls/rustls).

`Cargo.toml:`
```
[dependencies]
crates_io_api = { version = "?", default-features = false, features = ["rustls"] }
```
