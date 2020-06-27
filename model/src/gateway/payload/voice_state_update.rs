use crate::voice::VoiceState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct VoiceStateUpdate(pub VoiceState);

#[cfg(test)]
mod tests {
    use super::{VoiceState, VoiceStateUpdate};
    use crate::{
        guild::Member,
        id::{GuildId, RoleId, UserId},
        user::User,
    };
    use serde_test::Token;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_voice_state_update() {
        let update = VoiceStateUpdate(VoiceState {
            channel_id: None,
            deaf: false,
            guild_id: Some(GuildId(1)),
            member: Some(Member {
                deaf: false,
                guild_id: GuildId(1),
                hoisted_role: Some(RoleId(4)),
                joined_at: None,
                mute: false,
                nick: None,
                premium_since: None,
                roles: vec![RoleId(4)],
                user: User {
                    id: UserId(1),
                    avatar: None,
                    bot: false,
                    discriminator: "0909".to_string(),
                    name: "foo".to_string(),
                    mfa_enabled: None,
                    locale: None,
                    verified: None,
                    email: None,
                    flags: None,
                    premium_type: None,
                    system: None,
                    public_flags: None,
                },
            }),
            mute: false,
            self_deaf: false,
            self_mute: false,
            self_stream: false,
            session_id: "a".to_owned(),
            suppress: false,
            token: None,
            user_id: UserId(1),
        });

        serde_test::assert_tokens(
            &update,
            &[
                Token::NewtypeStruct {
                    name: "VoiceStateUpdate",
                },
                Token::Struct {
                    name: "VoiceState",
                    len: 12,
                },
                Token::Str("channel_id"),
                Token::None,
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("1"),
                Token::Str("member"),
                Token::Some,
                Token::Struct {
                    name: "Member",
                    len: 9,
                },
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("guild_id"),
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("1"),
                Token::Str("hoisted_role"),
                Token::Some,
                Token::NewtypeStruct { name: "RoleId" },
                Token::Str("4"),
                Token::Str("joined_at"),
                Token::None,
                Token::Str("mute"),
                Token::Bool(false),
                Token::Str("nick"),
                Token::None,
                Token::Str("premium_since"),
                Token::None,
                Token::Str("roles"),
                Token::Seq { len: Some(1) },
                Token::NewtypeStruct { name: "RoleId" },
                Token::Str("4"),
                Token::SeqEnd,
                Token::Str("user"),
                Token::Struct {
                    name: "User",
                    len: 13,
                },
                Token::Str("id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("1"),
                Token::Str("avatar"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0909"),
                Token::Str("username"),
                Token::Str("foo"),
                Token::Str("mfa_enabled"),
                Token::None,
                Token::Str("locale"),
                Token::None,
                Token::Str("verified"),
                Token::None,
                Token::Str("email"),
                Token::None,
                Token::Str("flags"),
                Token::None,
                Token::Str("premium_type"),
                Token::None,
                Token::Str("system"),
                Token::None,
                Token::Str("public_flags"),
                Token::None,
                Token::StructEnd,
                Token::StructEnd,
                Token::Str("mute"),
                Token::Bool(false),
                Token::Str("self_deaf"),
                Token::Bool(false),
                Token::Str("self_mute"),
                Token::Bool(false),
                Token::Str("self_stream"),
                Token::Bool(false),
                Token::Str("session_id"),
                Token::Str("a"),
                Token::Str("suppress"),
                Token::Bool(false),
                Token::Str("token"),
                Token::None,
                Token::Str("user_id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("1"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn voice_state_update_deser() {
        let input = serde_json::json!({
            "member": {
                "user": {
                    "username": "Twilight Sparkle",
                    "id": "1234123123123",
                    "discriminator": "4242",
                    "avatar": "a21312321231236060dfe562c"
                },
                "roles": [
                    "123",
                    "124"
                ],
                "nick": "Twilight",
                "mute": false,
                "joined_at": "2016-12-08T18:41:21.954000+00:00",
                "hoisted_role": "123",
                "deaf": false
            },
            "user_id": "123213",
            "suppress": false,
            "session_id": "asdasdas1da98da2b3ab3a",
            "self_video": false,
            "self_mute": false,
            "self_deaf": false,
            "mute": false,
            "guild_id": "999999",
            "deaf": false,
            "channel_id": null
        });

        let expected = VoiceStateUpdate(VoiceState {
            channel_id: None,
            deaf: false,
            guild_id: Some(GuildId(999999)),
            member: Some(Member {
                deaf: false,
                guild_id: GuildId(999999),
                hoisted_role: Some(RoleId(123)),
                joined_at: Some("2016-12-08T18:41:21.954000+00:00".to_string()),
                mute: false,
                nick: Some("Twilight".to_string()),
                premium_since: None,
                roles: vec![RoleId(123), RoleId(124)],
                user: User {
                    id: UserId(1234123123123),
                    avatar: Some("a21312321231236060dfe562c".to_string()),
                    bot: false,
                    discriminator: "4242".to_string(),
                    name: "Twilight Sparkle".to_string(),
                    mfa_enabled: None,
                    locale: None,
                    verified: None,
                    email: None,
                    flags: None,
                    premium_type: None,
                    system: None,
                    public_flags: None,
                },
            }),
            mute: false,
            self_deaf: false,
            self_mute: false,
            self_stream: false,
            session_id: "asdasdas1da98da2b3ab3a".to_owned(),
            suppress: false,
            token: None,
            user_id: UserId(123213),
        });

        let parsed: VoiceStateUpdate = serde_json::from_value(input).unwrap();

        assert_eq!(parsed, expected);
    }
}
