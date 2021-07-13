//! # twilight-util
//!
//! [![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]
//!
//! `twilight-util` is a set of utility types and functions for the [`twilight-rs`] ecosystem to
//! augment or enhance default functionality.
//!
//! ## Features
//!
//! ### `permission-calculator`
//!
//! Allows the use of a calculator to determine the permissions of a member in
//! a guild or channel.
//!
//! ### `snowflake`
//!
//! Allows the use of the `Snowflake` trait, which provides methods for the extraction of
//! structured information from [Discord snowflakes].
//!
//! [`twilight-rs`]: https://github.com/twilight-rs/twilight
//! [discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
//! [discord link]: https://discord.gg/7jj8n7D
//! [github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
//! [github link]: https://github.com/twilight-rs/twilight
//! [license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
//! [license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
//! [rust badge]: https://img.shields.io/badge/rust-1.51+-93450a.svg?style=for-the-badge&logo=rust
//! [Discord snowflakes]: https://discord.com/developers/docs/reference#snowflakes

#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::pedantic,
    future_incompatible,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    unsafe_code,
    unused,
    warnings
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "link")]
#[cfg_attr(docsrs, doc(cfg(feature = "link")))]
pub mod link;

#[cfg(feature = "permission-calculator")]
#[cfg_attr(docsrs, doc(cfg(feature = "permission-calculator")))]
pub mod permission_calculator;

#[cfg(feature = "snowflake")]
#[cfg_attr(docsrs, doc(cfg(feature = "snowflake")))]
pub mod snowflake;
