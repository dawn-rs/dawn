<!-- cargo-sync-readme start -->

# twilight-gateway

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-gateway` is an implementation of Discord's sharding gateway sessions.
This is responsible for receiving stateful events in real-time from Discord
and sending *some* stateful information.

It includes two primary types: the Shard and Cluster.

The Shard handles a single websocket connection and can manage up to 2500
guilds. If you manage a small bot in under about 2000 guilds, then this is
what you use. See the [Discord docs][docs:discord:sharding] for more
information on sharding.

The Cluster is an interface which manages the health of the shards it
manages and proxies all of their events under one unified stream. This is
useful to use if you have a large bot in over 1000 or 2000 guilds.

## Features

### Deserialization

`twilight-gateway` supports [`serde_json`] and [`simd-json`] for
deserializing and serializing events.

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
you can also use this environment variable `RUSTFLAGS="-C target-cpu=native"`.

```toml
[dependencies]
twilight-gateway = { default-features = false, features = ["rustls", "simd-json"], version = "0.1" }
```

### TLS

`twilight-gateway` has features to enable [`async-tungstenite`] and
[`twilight-http`]'s TLS features. These features are mutually exclusive.
`rustls` is enabled by default.

#### `native`

The `native` feature enables [`async-tungstenite`]'s `tokio-native-tls`
feature as well as [`twilight-http`]'s `native` feature which is mostly
equivalent to using [`native-tls`].

To enable `native`, do something like this in your `Cargo.toml`:

```toml
[dependencies]
twilight-gateway = { default-features = false, features = ["native"], version = "0.1" }
```

#### `rustls`

The `rustls` feature enables [`async-tungstenite`]'s `async-tls` feature and
[`twilight-http`]'s `rustls` feature, which use [`rustls`] as the TLS backend.

This is enabled by default.

### zlib

The `stock-zlib` feature enables [`flate2`]'s `zlib` feature which makes
[`flate2`] use system zlib instead of [`zlib-ng`].

This is not enabled by default.

[`async-tungstenite`]: https://crates.io/crates/async-tungstenite
[`flate2`]: https://crates.io/crates/flate2
[`native-tls`]: https://crates.io/crates/native-tls
[`rustls`]: https://crates.io/crates/rustls
[`serde_json`]: https://crates.io/crates/serde_json
[`simd-json`]: https://crates.io/crates/simd-json
[`twilight-http`]: https://twilight-rs.github.io/twilight/twilight_http/index.html
[`zlib-ng`]: https://github.com/zlib-ng/zlib-ng
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[docs:discord:sharding]: https://discord.com/developers/docs/topics/gateway#sharding
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/trunk/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-stable-93450a.svg?style=for-the-badge&logo=rust

<!-- cargo-sync-readme end -->
