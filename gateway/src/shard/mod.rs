//! Types for working with and running connections to the gateway.
//!
//! At the heart of the `shard` module is the [`Shard`] itself: it's the
//! interface used to start a shard, send messages to the gateway, and receive
//! [`Event`]s from it, such as [new messages] or [channel deletions].
//!
//! Once running, the shard maintains [information about itself] that you can
//! obtain through it. This is information such as the latency or the current
//! [`Stage`] of the connection, like whether it's [`Disconnected`] or
//! [`Resuming`] the connection.
//!
//! Shards are configurable through the [`ShardConfigBuilder`] struct, which
//! provides a clean interface for correctly building a [`ShardConfig`].
//!
//! [`ShardConfig`]: config/struct.ShardConfig.html
//! [`ShardConfigBuilder`]: config/struct.ShardConfigBuilder.html
//! [`Event`]: event/enum.Event.html
//! [`Shard`]: struct.Shard.html
//! [`Stage`]: enum.Stage.html
//! [`Disconnected`]: enum.Stage.html#variant.Disconnected
//! [`Resuming`]: enum.Stage.html#variant.Resuming
//! [channel deletions]: event/enum.Event.html#variant.ChannelDelete
//! [new messages]: event/enum.Event.html#variant.MessageCreate

pub mod config;
pub mod error;
pub mod stage;

mod event;
mod r#impl;
mod processor;
mod sink;

pub use self::{
    config::ShardConfig,
    error::{Error, Result},
    event::Events,
    processor::heartbeat::Latency,
    r#impl::{Information, ResumeSession, Shard},
    sink::ShardSink,
    stage::Stage,
};

use async_tungstenite::{tokio::ConnectStream, WebSocketStream};

type ShardStream = WebSocketStream<ConnectStream>;
