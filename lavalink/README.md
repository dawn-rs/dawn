<!-- cargo-sync-readme start -->

# twilight-lavalink

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-lavalink` is a client for [Lavalink] as part of the twilight
ecosystem.

It includes support for managing multiple nodes, a player manager for
conveniently using players to send events and retrieve information for each
guild, and an HTTP module for creating requests using the [`http`] crate and
providing models to deserialize their responses. It will automatically
handle sending voice channel updates to Lavalink by processing events via
the [client's `process` method][`Lavalink::process`], which you must call
with every Voice State Update and Voice Server Update you receive.

## Features

### `http-support`

The `http-support` feature adds support for the `http` module to return
request types from the [`http`] crate. This is enabled by default.

### TLS

`twilight-lavalink` has features to enable [`async-tungstenite`]'s TLS
features. These features are mutually exclusive. `rustls` is enabled by
default.

#### `native`

The `native` feature enables [`async-tungstenite`]'s `tokio-native-tls`
feature.

To enable `native`, do something like this in your `Cargo.toml`:

```toml
[dependencies]
twilight-lavalink = { default-features = false, features = ["native"], version = "0.2" }
```

#### `rustls`

The `rustls` feature enables [`async-tungstenite`]'s `tokio-rustls` feature, which
use [`rustls`] as the TLS backend.

This is enabled by default.

## Examples

Create a [client], add a [node], and give events to the client to [process]
events:

```rust,no_run
use futures_util::stream::StreamExt;
use std::{
    convert::TryInto,
    env,
    error::Error,
    future::Future,
    net::SocketAddr,
    str::FromStr,
};
use twilight_gateway::{Event, Intents, Shard};
use twilight_http::Client as HttpClient;
use twilight_lavalink::{http::LoadedTracks, model::Play, Lavalink};
use twilight_model::{
    channel::Message,
    gateway::payload::MessageCreate,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let token = env::var("DISCORD_TOKEN")?;
    let lavalink_host = SocketAddr::from_str(&env::var("LAVALINK_HOST")?)?;
    let lavalink_auth = env::var("LAVALINK_AUTHORIZATION")?;
    let shard_count = 1u64;

    let http = HttpClient::new(&token);
    let user_id = http.current_user().exec().await?.model().await?.id;

    let lavalink = Lavalink::new(user_id, shard_count);
    lavalink.add(lavalink_host, lavalink_auth).await?;

    let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES;
    let (shard, mut events) = Shard::new(token, intents);
    shard.start().await?;

    while let Some(event) = events.next().await {
        lavalink.process(&event).await?;
    }

    Ok(())
}
```

[Lavalink]: https://github.com/freyacodes/Lavalink
[`async-tungstenite`]: https://crates.io/crates/async-tungstenite
[`http`]: https://crates.io/crates/http
[`rustls`]: https://crates.io/crates/rustls
[client]: Lavalink
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[node]: Node
[process]: Lavalink::process
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust

<!-- cargo-sync-readme end -->
