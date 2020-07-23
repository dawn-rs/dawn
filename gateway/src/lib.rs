//! # twilight-gateway
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
//! twilight-gateway = { branch = "trunk", default-features = false, features = ["rustls", "simd-json"], git = "https://github.com/twilight-rs/twilight" }
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
//! twilight-gateway = { branch = "trunk", default-features = false, features = ["native", "serde_json"], git = "https://github.com/twilight-rs/twilight" }
//! ```
//!
//! #### `rustls`
//!
//! The `rustls` feature enables [`async-tungstenite`]'s `async-tls` feature and
//! [`twilight-http`]'s `rustls` feature, which use [`rustls`] as the TLS backend.
//!
//! This is enabled by default.
//!
//! [`async-tungstenite`]: https://crates.io/crates/async-tungstenite
//! [`native-tls`]: https://crates.io/crates/native-tls
//! [`rustls`]: https://crates.io/crates/rustls
//! [`serde_json`]: https://crates.io/crates/serde_json
//! [`simd-json`]: https://crates.io/crates/simd-json
//! [`twilight-http`]: https://twilight-rs.github.io/twilight/twilight_http/index.html
//! [docs:discord:sharding]: https://discord.com/developers/docs/topics/gateway#sharding

#![deny(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unsafe_code
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod cluster;
pub mod queue;
pub mod shard;

mod event;
mod listener;

pub use self::{
    cluster::{Cluster, ClusterConfig},
    event::EventTypeFlags,
    shard::{Shard, ShardConfig},
};
pub use twilight_model::gateway::event::{Event, EventType};

#[cfg(not(feature = "simd-json"))]
pub(crate) use serde_json::to_vec as json_to_vec;
#[cfg(feature = "simd-json")]
pub(crate) use simd_json::to_vec as json_to_vec;

#[cfg(not(feature = "simd-json"))]
pub(crate) use serde_json::to_string as json_to_string;
#[cfg(feature = "simd-json")]
pub(crate) use simd_json::to_string as json_to_string;

#[cfg(not(any(feature = "native", feature = "rustls")))]
compile_error!("Either the `native` or `rustls` feature must be enabled");
