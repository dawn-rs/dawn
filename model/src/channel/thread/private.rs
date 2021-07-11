use crate::channel::{
    permission_overwrite::PermissionOverwrite,
    thread::{ThreadMember, ThreadMetadata},
    ChannelType,
};
use crate::id::{ChannelId, GuildId, MessageId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PrivateThread {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<ThreadMember>,
    /// Max value of 50.
    pub member_count: u8,
    /// Max value of 50.
    pub message_count: u8,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u64>,
    pub thread_metadata: ThreadMetadata,
}

#[cfg(test)]
mod tests {
    use super::{ChannelId, ChannelType, GuildId, MessageId, ThreadMember, ThreadMetadata, UserId};
    use crate::channel::thread::{AutoArchiveDuration, PrivateThread};
    use serde_test::Token;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_private_thread() {
        let value = PrivateThread {
            guild_id: Some(GuildId(2)),
            id: ChannelId(1),
            kind: ChannelType::GuildPrivateThread,
            last_message_id: Some(MessageId(5)),
            member: Some(ThreadMember {
                id: Some(ChannelId(10)),
                user_id: Some(UserId(11)),
                join_timestamp: "456".to_owned(),
                flags: 12,
            }),
            member_count: 7,
            message_count: 6,
            name: "test".to_owned(),
            owner_id: Some(UserId(3)),
            parent_id: Some(ChannelId(4)),
            rate_limit_per_user: Some(8),
            permission_overwrites: Vec::new(),
            thread_metadata: ThreadMetadata {
                archived: true,
                auto_archive_duration: AutoArchiveDuration::Hour,
                archive_timestamp: "123".to_string(),
                locked: true,
            },
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "PrivateThread",
                    len: 13,
                },
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("2"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "ChannelId" },
                Token::Str("1"),
                Token::Str("type"),
                Token::U8(12),
                Token::Str("last_message_id"),
                Token::Some,
                Token::NewtypeStruct { name: "MessageId" },
                Token::Str("5"),
                Token::Str("member"),
                Token::Some,
                Token::Struct {
                    name: "ThreadMember",
                    len: 4,
                },
                Token::Str("flags"),
                Token::U64(12),
                Token::Str("id"),
                Token::Some,
                Token::NewtypeStruct { name: "ChannelId" },
                Token::Str("10"),
                Token::Str("join_timestamp"),
                Token::Str("456"),
                Token::Str("user_id"),
                Token::Some,
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("11"),
                Token::StructEnd,
                Token::Str("member_count"),
                Token::U8(7),
                Token::Str("message_count"),
                Token::U8(6),
                Token::Str("name"),
                Token::Str("test"),
                Token::Str("owner_id"),
                Token::Some,
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("3"),
                Token::Str("parent_id"),
                Token::Some,
                Token::NewtypeStruct { name: "ChannelId" },
                Token::Str("4"),
                Token::Str("permission_overwrites"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("rate_limit_per_user"),
                Token::Some,
                Token::U64(8),
                Token::Str("thread_metadata"),
                Token::Struct {
                    name: "ThreadMetadata",
                    len: 4,
                },
                Token::Str("archived"),
                Token::Bool(true),
                Token::Str("auto_archive_duration"),
                Token::U16(60),
                Token::Str("archive_timestamp"),
                Token::Str("123"),
                Token::Str("locked"),
                Token::Bool(true),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
