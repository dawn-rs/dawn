# Changelog

Changelog for `twilight-cache-inmemory`.

## [0.2.0-beta.0] = 2020-10-10

This beta version of major version 0.2 of the crate includes changes needed to
support `twilight-model` 0.2.

### Changes

All changes in this version are from PR [#532].

The following model fields have been removed:
- `model::CachedGuild::{embed_channel_id, embed_enabled}`
- `model::CachedPresence::nick`

The crate now depends on version 0.2 of `twilight-model`.

## [0.1.1] - 2020-10-05

### Added

- Add methods to fetch guild relation ID sets ([#528] - [@Erk-])

## [0.1.0] - 2020-09-13

Initial release.

[@Erk-]: https://github.com/Erk-

[#532]: https://github.com/twilight-rs/twilight/pull/532
[#528]: https://github.com/twilight-rs/twilight/pull/528

[0.2.0-beta.0]: https://github.com/twilight-rs/twilight/releases/tag/cache-in-memory-v0.2.0-beta.0
[0.1.1]: https://github.com/twilight-rs/twilight/releases/tag/cache-in-memory-v0.1.1
[0.1.0]: https://github.com/twilight-rs/twilight/releases/tag/v0.1.0
