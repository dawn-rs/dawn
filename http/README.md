<!-- cargo-sync-readme start -->

# twilight-http

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

HTTP support for the twilight ecosystem.

## Features

### Deserialization

`twilight-http` supports [`serde_json`] and [`simd-json`] for deserializing
responses.

#### `simd-json`

The `simd-json` feature enables [`simd-json`] support to use simd features
of modern cpus to deserialize responses faster. It is not enabled by
default.

To use this feature you need to also add these lines to
`<project root>/.cargo/config`:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

You can also set the environment variable
`RUSTFLAGS="-C target-cpu=native"`. If you enable both `serde_json` and
`simd-json` at the same time, then `simd-json` will be used.

To enable `simd-json`, do something like this in your `Cargo.toml`:

```toml
[dependencies]
twilight-http = { default-features = false, features = ["rustls", "simd-json"], version = "0.2" }
```

### TLS

`twilight-http` has features to enable [`hyper`]'s TLS features. These
features are mutually exclusive. `rustls` is enabled by default.

#### `native`

The `native` feature enables [`hyper`]'s `default-tls`
feature, which is mostly equivalent to using [`native-tls`].

To enable `native`, do something like this in your `Cargo.toml`:

```toml
[dependencies]
twilight-http = { default-features = false, features = ["native"], version = "0.2" }
```

#### `rustls`

The `rustls` feature enables [`hyper`]'s `rustls` feature, which uses
[`rustls`] as the TLS backend.

This is enabled by default.

### Tracing

The `tracing` feature enables logging via the [`tracing`] crate.

This is enabled by default.

[`native-tls`]: https://crates.io/crates/native-tls
[`hyper`]: https://crates.io/crates/hyper
[`rustls`]: https://crates.io/crates/rustls
[`serde_json`]: https://crates.io/crates/serde_json
[`simd-json`]: https://crates.io/crates/simd-json
[`tracing`]: https://crates.io/crates/tracing
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust

<!-- cargo-sync-readme end -->
