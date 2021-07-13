<!-- cargo-sync-readme start -->

# twilight-embed-builder

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-embed-builder` is a set of builder for the [`twilight-rs`]
ecosystem to create a message embed, useful when creating or updating
messages.

## Examples

Build a simple embed:

```rust,no_run
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

let embed = EmbedBuilder::new()
    .description("Here's a list of reasons why Twilight is the best pony:")
    .field(EmbedFieldBuilder::new("Wings", "She has wings.").inline())
    .field(EmbedFieldBuilder::new("Horn", "She can do magic, and she's really good at it.").inline())
    .build()?;
```

Build an embed with an image:

```rust,no_run
use twilight_embed_builder::{EmbedBuilder, ImageSource};

let embed = EmbedBuilder::new()
    .description("Here's a cool image of Twilight Sparkle")
    .image(ImageSource::attachment("bestpony.png")?)
    .build()?;

```

[`twilight-rs`]: https://github.com/twilight-rs/twilight
[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust
[the discord docs]: https://discord.com/developers/docs/resources/channel#create-message-using-attachments-within-embeds

<!-- cargo-sync-readme end -->
