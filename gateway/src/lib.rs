//! # twilight-gateway
//!
//! [![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]
//!
//! `twilight-gateway` is an implementation of Discord's sharding gateway sessions.
//! This is responsible for receiving stateful events in real-time from Discord
//! and sending *some* stateful information.
//!
//! It includes two primary types: the Shard and Cluster.
//!
//! The Shard handles a single websocket connection and can manage up to 2500
//! guilds. If you manage a small bot in under about 2000 guilds, then this is
//! what you use. See the [Discord docs][docs:discord:sharding] for more
//! information on sharding.
//!
//! The Cluster is an interface which manages the health of the shards it
//! manages and proxies all of their events under one unified stream. This is
//! useful to use if you have a large bot in over 1000 or 2000 guilds.
//!
//! ## Features
//!
//! ### Deserialization
//!
//! `twilight-gateway` supports [`serde_json`] and [`simd-json`] for
//! deserializing and serializing events.
//!
//! #### `simd-json`
//!
//! The `simd-json` feature enables [`simd-json`] support to use simd features
//! of modern cpus to deserialize responses faster. It is not enabled by
//! default.
//!
//! To use this feature you need to also add these lines to
//! `<project root>/.cargo/config`:
//!
//! ```toml
//! [build]
//! rustflags = ["-C", "target-cpu=native"]
//! ```
//! you can also use this environment variable `RUSTFLAGS="-C target-cpu=native"`.
//!
//! ```toml
//! [dependencies]
//! twilight-gateway = { default-features = false, features = ["rustls", "simd-json"], version = "0.2" }
//! ```
//!
//! ### TLS
//!
//! `twilight-gateway` has features to enable [`async-tungstenite`] and
//! [`twilight-http`]'s TLS features. These features are mutually exclusive.
//! `rustls` is enabled by default.
//!
//! #### `native`
//!
//! The `native` feature enables [`async-tungstenite`]'s `tokio-native-tls`
//! feature as well as [`twilight-http`]'s `native` feature which is mostly
//! equivalent to using [`native-tls`].
//!
//! To enable `native`, do something like this in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! twilight-gateway = { default-features = false, features = ["native"], version = "0.2" }
//! ```
//!
//! #### `rustls`
//!
//! The `rustls` feature enables [`async-tungstenite`]'s `tokio-rustls` feature and
//! [`twilight-http`]'s `rustls` feature, which use [`rustls`] as the TLS backend.
//!
//! This is enabled by default.
//!
//! ### zlib
//!
//! zlib is enabled with the feature `compression` and one of the two `zlib` features
//! described below. Enabling any of the two features below will also enable
//! `compression`. `compression` is enabled by default.
//!
//! There are 2 zlib features `zlib-stock` and `zlib-simd` for the library to work
//! one of them has to be enabled. If both are enabled it will use `zlib-stock`
//!
//! `zlib-stock` enabled by default.
//!
//! Enabling **only** `zlib-simd` will make the library use [`zlib-ng`] which is a modern
//! fork of zlib that is faster and more effective, but it needs `cmake` to compile.
//!
//! ### Tracing
//!
//! The `tracing` feature enables logging via the [`tracing`] crate.
//!
//! This is enabled by default.
//!
//! ### Metrics
//!
//! The `metrics` feature provides metrics information via the `metrics` crate.
//! Some of the metrics logged are counters about received event counts and
//! their types and gauges about the capacity and efficiency of the inflater of
//! each shard.
//!
//! This is disabled by default.
//!
//! [`async-tungstenite`]: https://crates.io/crates/async-tungstenite
//! [`native-tls`]: https://crates.io/crates/native-tls
//! [`rustls`]: https://crates.io/crates/rustls
//! [`serde_json`]: https://crates.io/crates/serde_json
//! [`simd-json`]: https://crates.io/crates/simd-json
//! [`tracing`]: https://crates.io/crates/tracing
//! [`twilight-http`]: https://twilight-rs.github.io/twilight/twilight_http/index.html
//! [`zlib-ng`]: https://github.com/zlib-ng/zlib-ng
//! [discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
//! [discord link]: https://discord.gg/7jj8n7D
//! [docs:discord:sharding]: https://discord.com/developers/docs/topics/gateway#sharding
//! [github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
//! [github link]: https://github.com/twilight-rs/twilight
//! [license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
//! [license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
//! [rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust

#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::pedantic,
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    broken_intra_doc_links,
    unused,
    warnings
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::used_underscore_binding
)]

pub mod cluster;
pub mod shard;

mod event;

pub use self::event::EventTypeFlags;
pub use twilight_model::gateway::Intents;

#[doc(no_inline)]
pub use self::{
    cluster::{Cluster, Config as ClusterConfig},
    shard::{Config as ShardConfig, Shard},
};
#[doc(no_inline)]
pub use twilight_gateway_queue as queue;
#[doc(no_inline)]
pub use twilight_model::gateway::event::{Event, EventType};

#[cfg(not(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
)))]
compile_error!(
    "Either the `native`, `rustls-native-roots` or `rustls-webpki-roots` feature must be enabled."
);
