<!-- cargo-sync-readme start -->

# twilight-command-parser

[![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]

`twilight-command-parser` is a command parser for the [`twilight-rs`]
ecosystem.

Included is a mutable configuration that allows you to specify the command
names and prefixes. The parser parses out commands matching an available
command and prefix and provides the command arguments to you.

### Examples

A simple parser for a bot with one prefix (`"!"`) and two commands: `"echo"`
and `"ping"`:

```rust,no_run
use twilight_command_parser::{Command, CommandParserConfig, Parser};

let mut config = CommandParserConfig::new();

config.add_command("echo", false);
config.add_command("ping", false);

// Add the prefix `"!"`.
// (Use `CommandParserConfig::add_prefixes` to add multiple prefixes)
config.add_prefix("!");

let parser = Parser::new(config);

// Now pass a command to the parser
match parser.parse("!echo a message") {
    Some(Command { name: "echo", arguments, .. }) => {
        let content = arguments.as_str();

        println!("Got an echo request to send `{}`", content);
    },
    Some(Command { name: "ping", .. }) => {
        println!("Got a ping request");
    },
    // Ignore all other commands.
    Some(_) => {},
    None => println!("Message didn't match a prefix and command"),
}
```

[discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
[discord link]: https://discord.gg/7jj8n7D
[github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
[github link]: https://github.com/twilight-rs/twilight
[license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
[license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
[rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust
[`twilight-rs`]: https://github.com/twilight-rs/twilight

<!-- cargo-sync-readme end -->
