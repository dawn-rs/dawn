<!-- cargo-sync-readme start -->

# twilight

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

![project logo][logo]

`twilight` is a powerful, flexible, and scalable ecosystem of Rust libraries
for the Discord API.

The ecosystem of first-class crates includes [`twilight-cache-inmemory`],
[`twilight-command-parser`], [`twilight-gateway`], [`twilight-http`],
[`twilight-model`], and more. These are explained in detail below.

The main `twilight` crate is purely an advertisement crate: it has *no*
functionality. Please use the individual crates listed below instead!

## Installation

Twilight supports a MSRV of Rust 1.51+.

We recommend that most users start out with these crates added to your
`Cargo.toml`'s `[dependencies]` section:

```toml
twilight-cache-inmemory = "0.4"
twilight-gateway = "0.4"
twilight-http = "0.4"
twilight-model = "0.4"
```

If you need any other functionality that Twilight provides, you can just add
that dependency in.

## Core Crates

These are essential crates that most users will use together for a full
development experience. You may not need all of these - such as
[`twilight-command-parser`] - but they are often used together to accomplish
most of what you need.

### [`twilight-model`]

Models defining structures, enums, and bitflags for the entirety of the
Discord API. It is split into a number of sub-modules, such as `gateway` for
containing the WebSocket gateway types, `guild` for containing types owned
by guilds (servers), `voice` containing the types used by the Voice
WebSocket API, and more.

These are all in a single crate so that you can use `gateway` models without
depending on [`twilight-gateway`]. One use case is if you write your own
WebSocket gateway implementation.

### [`twilight-cache-inmemory`]

In-process-memory based cache over objects received from the gateway. It's
responsible for holding and managing information about things like guilds,
channels, role information, voice states, and any other events that come
from Discord.

### [`twilight-gateway`]

Implementation of Discord's sharding gateway sessions. This is responsible
for receiving stateful events in real-time from Discord and sending *some*
stateful information.

### [`twilight-command-parser`]

Helpful crate for parsing commands out of messages received over the
gateway. It finds messages commanding your bot and parses the arguments out.

### [`twilight-http`]

HTTP client supporting all of the Discord REST API. It is based on `hyper`.
It meets Discord's ratelimiting requirements and supports proxying.

### [`twilight-standby`]

Event processor that allows for tasks to wait for an event to come in. This
is useful, for example, when you have a reaction menu and want to wait for a
specific reaction on it to come in.

## Additional Crates

These are crates that are officially supported by Twilight, but aren't
considered core crates due to being vendor-specific or non-essential for
most users.

### [`twilight-embed-builder`]

Utility crate for creating and validating message embeds, to be used when
creating or updating messages.

### [`twilight-lavalink`]

Client for [Lavalink] as part of the twilight ecosystem.

It includes support for managing multiple nodes, a player manager for
conveniently using players to send events and retrieve information for each
guild, and an HTTP module for creating requests using the [`http`] crate and
providing models to deserialize their responses.

### [`twilight-mention`]

Create display formatters for various model types that format mentions. For
example, it can create formatters for mentioning a channel or emoji, or
pinging a role or user.

### [`twilight-util`]

Utility crate that adds utilities to the twilight ecosystem that do not fit
in any other crate. Currently, it contains:

- A trait to make extracting data from Discord identifiers (Snowflakes)
easier;
- A calculator to calculate the permissions of a member in a guild or
channel.

### [`twilight-gateway-queue`]

A trait and some implementations that are used by the gateway to ratelimit
identify calls. Developers should prefer to use the re-exports of these
crates through the gateway.

## Examples

```rust,no_run
use std::{env, error::Error};
use futures::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(&token, Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;

    // Start up the cluster.
    let cluster_spawn = cluster.clone();

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = HttpClient::new(&token);

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, http.clone()));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: HttpClient,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .exec()
                .await?;
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
```

## Note about tracing

When using the `tracing` crate you won't, by default, see logs from any
libraries that use the `log` crate. You can add that back by using the
[`tracing-log`] crate and initializing it like this:

```rust
tracing_log::LogTracer::init()?;
```

## License

All first-party crates are licensed under [ISC][LICENSE.md]

[LICENSE.md]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[Lavalink]: https://github.com/freyacodes/Lavalink
[`http`]: https://crates.io/crates/http
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[docs:discord:sharding]: https://discord.com/developers/docs/topics/gateway#sharding
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[logo]: https://raw.githubusercontent.com/twilight-rs/twilight/main/logo.png
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust
[`tracing-log`]: https://github.com/tokio-rs/tracing/tree/master/tracing-log
[`twilight-cache-inmemory`]: https://twilight.rs/chapter_1_crates/section_4_cache_inmemory.html
[`twilight-command-parser`]: https://twilight.rs/chapter_1_crates/section_5_command_parser.html
[`twilight-embed-builder`]: https://twilight.rs/chapter_1_crates/section_7_first_party/section_1_embed_builder.html
[`twilight-gateway-queue`]: https://twilight.rs/chapter_1_crates/section_7_first_party/section_5_gateway_queue.html
[`twilight-gateway`]: https://twilight.rs/chapter_1_crates/section_3_gateway.html
[`twilight-http`]: https://twilight.rs/chapter_1_crates/section_2_http.html
[`twilight-lavalink`]: https://twilight.rs/chapter_1_crates/section_7_first_party/section_3_lavalink.html
[`twilight-mention`]: https://twilight.rs/chapter_1_crates/section_7_first_party/section_2_mention.html
[`twilight-model`]: https://twilight.rs/chapter_1_crates/section_1_model.html
[`twilight-standby`]: https://twilight.rs/chapter_1_crates/section_6_standby.html
[`twilight-util`]: https://twilight.rs/chapter_1_crates/section_7_first_party/section_4_util.html

<!-- cargo-sync-readme end -->
