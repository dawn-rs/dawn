mod activity;
mod activity_assets;
mod activity_flags;
mod activity_party;
mod activity_secrets;
mod activity_timestamps;
mod activity_type;
mod client_status;
mod status;

pub use self::{
    activity::Activity,
    activity_assets::ActivityAssets,
    activity_flags::ActivityFlags,
    activity_party::ActivityParty,
    activity_secrets::ActivitySecrets,
    activity_timestamps::ActivityTimestamps,
    activity_type::ActivityType,
    client_status::ClientStatus,
    status::Status,
};

use crate::{
    id::{GuildId, UserId},
    user::User,
};

#[cfg_attr(
    feature = "serde-support",
    derive(serde::Deserialize, serde::Serialize)
)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Presence {
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub activities: Vec<Activity>,
    pub client_status: ClientStatus,
    pub game: Option<Activity>,
    pub guild_id: Option<GuildId>,
    pub nick: Option<String>,
    pub status: Status,
    pub user: UserOrId,
}

#[cfg_attr(
    feature = "serde-support",
    derive(serde::Deserialize, serde::Serialize)
)]
#[cfg_attr(feature = "serde-support", serde(untagged))]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UserOrId {
    User(User),
    UserId { id: UserId },
}

#[cfg(feature = "serde-support")]
mod serde_support {
    use super::{Presence, UserOrId};
    use crate::id::UserId;
    use serde_mappable_seq::Key;

    impl Key<'_, UserId> for Presence {
        fn key(&self) -> UserId {
            match self.user {
                UserOrId::User(ref u) => u.id,
                UserOrId::UserId {
                    id,
                } => id,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Activity, ActivityType, ClientStatus, Presence, Status, UserOrId};
    use crate::id::UserId;
    use serde_test::Token;

    #[test]
    fn test_custom() {
        let activity = Activity {
            application_id: None,
            assets: None,
            created_at: Some(1571048061237),
            details: None,
            flags: None,
            id: Some("aaaaaaaaaaaaaaaa".to_owned()),
            instance: None,
            kind: ActivityType::Custom,
            name: "foo".to_owned(),
            party: None,
            secrets: None,
            state: None,
            timestamps: None,
            url: None,
        };
        let presence = Presence {
            activities: vec![activity.clone()],
            client_status: ClientStatus {
                desktop: Some(Status::Online),
                mobile: None,
                web: None,
            },
            game: Some(activity),
            guild_id: None,
            nick: None,
            status: Status::Online,
            user: UserOrId::UserId {
                id: UserId(1),
            },
        };

        serde_test::assert_de_tokens(
            &presence,
            &[
                Token::Struct {
                    name: "Presence",
                    len: 4,
                },
                Token::Str("user"),
                Token::Struct {
                    name: "UserOrId",
                    len: 1,
                },
                Token::Str("id"),
                Token::Str("1"),
                Token::StructEnd,
                Token::Str("status"),
                Token::Enum {
                    name: "Status",
                },
                Token::Str("online"),
                Token::Unit,
                Token::Str("game"),
                Token::Some,
                Token::Struct {
                    name: "Activity",
                    len: 4,
                },
                Token::Str("type"),
                Token::U8(4),
                Token::Str("name"),
                Token::Str("foo"),
                Token::Str("id"),
                Token::Some,
                Token::Str("aaaaaaaaaaaaaaaa"),
                Token::Str("created_at"),
                Token::Some,
                Token::U64(1571048061237),
                Token::StructEnd,
                Token::Str("client_status"),
                Token::Struct {
                    name: "ClientStatus",
                    len: 3,
                },
                Token::Str("desktop"),
                Token::Some,
                Token::Enum {
                    name: "Status",
                },
                Token::Str("online"),
                Token::Unit,
                Token::Str("mobile"),
                Token::None,
                Token::Str("web"),
                Token::None,
                Token::StructEnd,
                Token::Str("activities"),
                Token::Seq {
                    len: Some(1),
                },
                Token::Struct {
                    name: "Activity",
                    len: 4,
                },
                Token::Str("type"),
                Token::U8(4),
                Token::Str("name"),
                Token::Str("foo"),
                Token::Str("id"),
                Token::Some,
                Token::Str("aaaaaaaaaaaaaaaa"),
                Token::Str("created_at"),
                Token::Some,
                Token::U64(1571048061237),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
