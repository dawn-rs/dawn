use crate::{
    id::{GuildId, RoleId},
    user::User,
};

use serde::{
    de::{
        value::MapAccessDeserializer, DeserializeSeed, Deserializer, Error as DeError, MapAccess,
        SeqAccess, Visitor,
    },
    Deserialize, Serialize,
};
use std::fmt::{Formatter, Result as FmtResult};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Member {
    pub deaf: bool,
    pub guild_id: GuildId,
    pub hoisted_role: Option<RoleId>,
    pub joined_at: Option<String>,
    pub mute: bool,
    pub nick: Option<String>,
    /// Whether the user has yet to pass the guild's [Membership Screening]
    /// requirements.
    pub pending: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

/// Version of [`Member`] but without a guild ID, useful in some contexts.
///
/// The HTTP and Gateway APIs don't include guild IDs in their payloads, so this
/// can be useful when you're unable to use a deserialization seed like
/// [`MemberDeserializer`].
// Used in the guild deserializer.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MemberIntermediary {
    pub deaf: bool,
    pub hoisted_role: Option<RoleId>,
    pub joined_at: Option<String>,
    pub mute: bool,
    pub nick: Option<String>,
    #[serde(default)]
    pub pending: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<String>,
    pub roles: Vec<RoleId>,
    pub user: User,
}

/// Deserialize a member when the payload doesn't have the guild ID but
/// you already know the guild ID.
///
/// Member payloads from the HTTP API, for example, don't have the guild
/// ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberDeserializer(GuildId);

impl MemberDeserializer {
    /// Create a new deserializer for a member when you know the ID but the
    /// payload probably doesn't contain it.
    pub const fn new(guild_id: GuildId) -> Self {
        Self(guild_id)
    }
}

impl<'de> DeserializeSeed<'de> for MemberDeserializer {
    type Value = Member;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(MemberVisitor(self.0))
    }
}

pub(crate) struct MemberVisitor(GuildId);

impl<'de> Visitor<'de> for MemberVisitor {
    type Value = Member;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a map of member fields")
    }

    fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
        let deser = MapAccessDeserializer::new(map);
        let member = MemberIntermediary::deserialize(deser)?;

        Ok(Member {
            deaf: member.deaf,
            guild_id: self.0,
            hoisted_role: member.hoisted_role,
            joined_at: member.joined_at,
            mute: member.mute,
            nick: member.nick,
            pending: member.pending,
            premium_since: member.premium_since,
            roles: member.roles,
            user: member.user,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OptionalMemberDeserializer(GuildId);

impl OptionalMemberDeserializer {
    /// Create a new deserializer for a member when you know the ID but the
    /// payload probably doesn't contain it.
    pub const fn new(guild_id: GuildId) -> Self {
        Self(guild_id)
    }
}

impl<'de> DeserializeSeed<'de> for OptionalMemberDeserializer {
    type Value = Option<Member>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_option(OptionalMemberVisitor(self.0))
    }
}

struct OptionalMemberVisitor(GuildId);

impl<'de> Visitor<'de> for OptionalMemberVisitor {
    type Value = Option<Member>;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("an optional member")
    }

    fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        Ok(Some(deserializer.deserialize_map(MemberVisitor(self.0))?))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberListDeserializer(GuildId);

impl MemberListDeserializer {
    /// Create a new deserializer for a map of members when you know the
    /// Guild ID but the payload probably doesn't contain it.
    pub const fn new(guild_id: GuildId) -> Self {
        Self(guild_id)
    }
}

struct MemberListVisitor(GuildId);

impl<'de> Visitor<'de> for MemberListVisitor {
    type Value = Vec<Member>;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a sequence of members")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut list = seq.size_hint().map_or_else(Vec::new, Vec::with_capacity);

        while let Some(member) = seq.next_element_seed(MemberDeserializer(self.0))? {
            list.push(member);
        }

        Ok(list)
    }
}

impl<'de> DeserializeSeed<'de> for MemberListDeserializer {
    type Value = Vec<Member>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(MemberListVisitor(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::Member;
    use crate::{
        id::{GuildId, RoleId, UserId},
        user::User,
    };
    use serde_test::Token;

    #[test]
    fn test_member_deserializer() {
        let value = Member {
            deaf: false,
            guild_id: GuildId(1),
            hoisted_role: Some(RoleId(2)),
            joined_at: Some("timestamp".to_owned()),
            mute: true,
            nick: Some("twilight".to_owned()),
            pending: false,
            premium_since: Some("timestamp".to_owned()),
            roles: Vec::new(),
            user: User {
                avatar: None,
                bot: false,
                discriminator: "0001".to_owned(),
                email: None,
                flags: None,
                id: UserId(3),
                locale: None,
                mfa_enabled: None,
                name: "twilight".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            },
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Member",
                    len: 10,
                },
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("1"),
                Token::Str("hoisted_role"),
                Token::Some,
                Token::NewtypeStruct { name: "RoleId" },
                Token::Str("2"),
                Token::Str("joined_at"),
                Token::Some,
                Token::Str("timestamp"),
                Token::Str("mute"),
                Token::Bool(true),
                Token::Str("nick"),
                Token::Some,
                Token::Str("twilight"),
                Token::Str("pending"),
                Token::Bool(false),
                Token::Str("premium_since"),
                Token::Some,
                Token::Str("timestamp"),
                Token::Str("roles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("user"),
                Token::Struct {
                    name: "User",
                    len: 5,
                },
                Token::Str("avatar"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0001"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("3"),
                Token::Str("username"),
                Token::Str("twilight"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
