use crate::guild::Member;
use std::ops::{Deref, DerefMut};

#[cfg_attr(
    feature = "serde-support",
    derive(serde::Deserialize, serde::Serialize)
)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MemberAdd(pub Member);

impl Deref for MemberAdd {
    type Target = Member;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MemberAdd {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Member, MemberAdd};
    use crate::{
        id::{GuildId, UserId},
        user::User,
    };
    use serde_test::Token;

    #[test]
    fn test_member_add() {
        let member_add = MemberAdd(Member {
            deaf: false,
            guild_id: Some(GuildId(1)),
            hoisted_role: None,
            joined_at: None,
            mute: false,
            nick: None,
            premium_since: None,
            roles: vec![],
            user: User {
                id: UserId(2),
                avatar: None,
                bot: false,
                discriminator: "0987".to_string(),
                name: "ab".to_string(),
            },
        });

        serde_test::assert_tokens(
            &member_add,
            &[
                Token::NewtypeStruct {
                    name: "MemberAdd",
                },
                Token::Struct {
                    name: "Member",
                    len: 9,
                },
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct {
                    name: "GuildId",
                },
                Token::Str("1"),
                Token::Str("hoisted_role"),
                Token::None,
                Token::Str("joined_at"),
                Token::None,
                Token::Str("mute"),
                Token::Bool(false),
                Token::Str("nick"),
                Token::None,
                Token::Str("premium_since"),
                Token::None,
                Token::Str("roles"),
                Token::Seq {
                    len: Some(0),
                },
                Token::SeqEnd,
                Token::Str("user"),
                Token::Struct {
                    name: "User",
                    len: 5,
                },
                Token::Str("id"),
                Token::NewtypeStruct {
                    name: "UserId",
                },
                Token::Str("2"),
                Token::Str("avatar"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0987"),
                Token::Str("username"),
                Token::Str("ab"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
