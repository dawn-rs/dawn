<!-- cargo-sync-readme start -->

# twilight-cache-inmemory

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-cache-inmemory` is an in-process-memory cache for the
[`twilight-rs`] ecosystem. It's responsible for processing events and
caching things like guilds, channels, users, and voice states.

## Examples

Update a cache with events that come in through the gateway:

```rust,no_run
use std::env;
use futures::stream::StreamExt;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Intents, Shard};

let token = env::var("DISCORD_TOKEN")?;
let (shard, mut events) = Shard::new(token, Intents::GUILD_MESSAGES);
shard.start().await?;

// Create a cache, caching up to 10 messages per channel:
let cache = InMemoryCache::builder().message_cache_size(10).build();

while let Some(event) = events.next().await {
    // Update the cache with the event.
    cache.update(&event);
}
```

## License

All first-party crates are licensed under [ISC][LICENSE.md]

[LICENSE.md]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[docs:discord:sharding]: https://discord.com/developers/docs/topics/gateway#sharding
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust

<!-- cargo-sync-readme end -->
