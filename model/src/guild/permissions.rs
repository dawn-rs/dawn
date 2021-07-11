use bitflags::bitflags;
use serde::{
    de::{Deserialize, Deserializer, Error as DeError, Visitor},
    ser::{Serialize, Serializer},
};
use std::fmt::{Formatter, Result as FmtResult};

bitflags! {
    pub struct Permissions: u64 {
        const CREATE_INVITE = 0x0000_0001;
        const KICK_MEMBERS = 0x0000_0002;
        const BAN_MEMBERS = 0x0000_0004;
        const ADMINISTRATOR = 0x0000_0008;
        const MANAGE_CHANNELS = 0x0000_0010;
        const MANAGE_GUILD = 0x0000_0020;
        const ADD_REACTIONS = 0x0000_0040;
        const VIEW_AUDIT_LOG = 0x0000_0080;
        const PRIORITY_SPEAKER = 0x0000_0100;
        const STREAM = 0x0000_0200;
        const VIEW_CHANNEL = 0x0000_0400;
        const SEND_MESSAGES = 0x0000_0800;
        const SEND_TTS_MESSAGES = 0x0000_1000;
        const MANAGE_MESSAGES = 0x0000_2000;
        const EMBED_LINKS = 0x0000_4000;
        const ATTACH_FILES = 0x0000_8000;
        const READ_MESSAGE_HISTORY = 0x0001_0000;
        const MENTION_EVERYONE = 0x0002_0000;
        const USE_EXTERNAL_EMOJIS = 0x0004_0000;
        const VIEW_GUILD_INSIGHTS = 0x0008_0000;
        const CONNECT = 0x0010_0000;
        const SPEAK = 0x0020_0000;
        const MUTE_MEMBERS = 0x0040_0000;
        const DEAFEN_MEMBERS = 0x0080_0000;
        const MOVE_MEMBERS = 0x0100_0000;
        const USE_VAD = 0x0200_0000;
        const CHANGE_NICKNAME = 0x0400_0000;
        const MANAGE_NICKNAMES = 0x0800_0000;
        const MANAGE_ROLES = 0x1000_0000;
        const MANAGE_WEBHOOKS = 0x2000_0000;
        const MANAGE_EMOJIS = 0x4000_0000;
        const USE_SLASH_COMMANDS = 0x8000_0000;
        const REQUEST_TO_SPEAK = 0x10000_0000;
        /// Allows for deleting and archiving threads, and viewing all private threads.
        const MANAGE_THREADS = 0x40000_0000;
        /// Allows for creating and participating in public threads.
        const USE_PUBLIC_THREADS = 0x0008_0000_0000;
        /// Allows for creating and participating in private threads.
        const USE_PRIVATE_THREADS = 0x0010_0000_0000;
    }
}

struct PermissionsVisitor;

impl<'de> Visitor<'de> for PermissionsVisitor {
    type Value = Permissions;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("integer or string permissions")
    }

    fn visit_u64<E: DeError>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Permissions::from_bits_truncate(v))
    }

    fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
        #[allow(clippy::map_err_ignore)]
        let num = v
            .parse()
            .map_err(|_| DeError::custom("permissions is not valid bitflags"))?;

        self.visit_u64(num)
    }
}

impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PermissionsVisitor)
    }
}

impl Serialize for Permissions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.bits().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Permissions;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_impl_all, const_assert_eq};
    use std::{fmt::Debug, hash::Hash};

    assert_impl_all!(
        Permissions: Copy,
        Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Ord,
        Send,
        Serialize,
        Sync
    );
    const_assert_eq!(0x8_0000_0000, Permissions::USE_PUBLIC_THREADS.bits());
    const_assert_eq!(0x10_0000_0000, Permissions::USE_PRIVATE_THREADS.bits());

    #[test]
    fn test_permissions() {
        let permissions = Permissions::DEAFEN_MEMBERS;

        serde_test::assert_tokens(&permissions, &[Token::Str("8388608")]);
    }
}
