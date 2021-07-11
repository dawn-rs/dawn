pub mod embed;
pub mod message;
pub mod permission_overwrite;
pub mod stage_instance;
pub mod thread;
pub mod webhook;

mod attachment;
mod category_channel;
mod channel_mention;
mod channel_type;
mod followed_channel;
mod group;
mod private_channel;
mod reaction;
mod reaction_type;
mod text_channel;
mod video_quality_mode;
mod voice_channel;

pub use self::{
    attachment::Attachment,
    category_channel::CategoryChannel,
    channel_mention::ChannelMention,
    channel_type::ChannelType,
    followed_channel::FollowedChannel,
    group::Group,
    message::Message,
    private_channel::PrivateChannel,
    reaction::Reaction,
    reaction_type::ReactionType,
    stage_instance::StageInstance,
    text_channel::TextChannel,
    video_quality_mode::VideoQualityMode,
    voice_channel::VoiceChannel,
    webhook::{Webhook, WebhookType},
};

use crate::{
    channel::thread::{
        AutoArchiveDuration, NewsThread, PrivateThread, PublicThread, ThreadMember, ThreadMetadata,
    },
    id::{ChannelId, GuildId, MessageId, UserId},
};
use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConversionError {
    MessageType(u8),
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConversionError::MessageType(num) => {
                f.write_str("Could not convert ")?;
                Display::fmt(num, f)?;

                f.write_str(" into a valid MessageType!")
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Channel {
    Group(Group),
    Guild(GuildChannel),
    Private(PrivateChannel),
}

impl Channel {
    /// Return the ID of the inner channel.
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::Group(group) => group.id,
            Self::Guild(guild_channel) => guild_channel.id(),
            Self::Private(private) => private.id,
        }
    }

    /// Return an immutable reference to the name of the inner channel.
    ///
    /// The group variant might not always have a name, since they are optional
    /// for groups. The guild variant will always have a name. The private
    /// variant doesn't have a name.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Group(group) => group.name.as_deref(),
            Self::Guild(guild_channel) => Some(guild_channel.name()),
            Self::Private(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum GuildChannel {
    Category(CategoryChannel),
    NewsThread(NewsThread),
    PrivateThread(PrivateThread),
    PublicThread(PublicThread),
    Text(TextChannel),
    Voice(VoiceChannel),
    Stage(VoiceChannel),
}

impl GuildChannel {
    /// Return the guild ID of the inner guild channel.
    pub const fn guild_id(&self) -> Option<GuildId> {
        match self {
            Self::Category(category) => category.guild_id,
            Self::NewsThread(thread) => thread.guild_id,
            Self::PrivateThread(thread) => thread.guild_id,
            Self::PublicThread(thread) => thread.guild_id,
            Self::Text(text) => text.guild_id,
            Self::Voice(voice) => voice.guild_id,
            Self::Stage(stage) => stage.guild_id,
        }
    }

    /// Return the ID of the inner guild channel.
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::Category(category) => category.id,
            Self::NewsThread(thread) => thread.id,
            Self::PrivateThread(thread) => thread.id,
            Self::PublicThread(thread) => thread.id,
            Self::Text(text) => text.id,
            Self::Voice(voice) => voice.id,
            Self::Stage(stage) => stage.id,
        }
    }

    /// Return an immutable reference to the name of the inner guild channel.
    pub fn name(&self) -> &str {
        match self {
            Self::Category(category) => category.name.as_ref(),
            Self::NewsThread(thread) => thread.name.as_ref(),
            Self::PrivateThread(thread) => thread.name.as_ref(),
            Self::PublicThread(thread) => thread.name.as_ref(),
            Self::Text(text) => text.name.as_ref(),
            Self::Voice(voice) => voice.name.as_ref(),
            Self::Stage(stage) => stage.name.as_ref(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum GuildChannelField {
    Bitrate,
    DefaultAutoArchiveDuration,
    GuildId,
    Id,
    LastMessageId,
    LastPinTimestamp,
    Member,
    MemberCount,
    MessageCount,
    Name,
    Nsfw,
    OwnerId,
    ParentId,
    PermissionOverwrites,
    Position,
    RateLimitPerUser,
    ThreadMetadata,
    Topic,
    Type,
    UserLimit,
    VideoQualityMode,
}

struct GuildChannelVisitor;

impl<'de> Visitor<'de> for GuildChannelVisitor {
    type Value = GuildChannel;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("enum GuildChannel")
    }

    #[allow(clippy::too_many_lines)]
    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        const VARIANTS: &[&str] = &[
            "GuildCategory",
            "GuildNews",
            "GuildNewsThread",
            "GuildPrivateThread",
            "GuildPublicThread",
            "GuildStore",
            "GuildText",
            "GuildVoice",
        ];

        let mut bitrate = None;
        let mut default_auto_archive_duration: Option<Option<AutoArchiveDuration>> = None;
        let mut guild_id = None;
        let mut id = None;
        let mut kind = None;
        let mut last_message_id: Option<Option<MessageId>> = None;
        let mut last_pin_timestamp: Option<Option<String>> = None;
        let mut member: Option<Option<ThreadMember>> = None;
        let mut member_count: Option<u8> = None;
        let mut message_count: Option<u8> = None;
        let mut name = None;
        let mut nsfw = None;
        let mut owner_id: Option<Option<UserId>> = None;
        let mut parent_id: Option<Option<ChannelId>> = None;
        let mut permission_overwrites = None;
        let mut position = None;
        let mut rate_limit_per_user = None;
        let mut thread_metadata: Option<ThreadMetadata> = None;
        let mut topic: Option<Option<String>> = None;
        let mut user_limit = None;
        let mut video_quality_mode = None;

        let span = tracing::trace_span!("deserializing guild channel");
        let _span_enter = span.enter();

        loop {
            let span_child = tracing::trace_span!("iterating over element");
            let _span_child_enter = span_child.enter();

            let key = match map.next_key() {
                Ok(Some(key)) => {
                    tracing::trace!(?key, "found key");

                    key
                }
                Ok(None) => break,
                Err(why) => {
                    // Encountered when we run into an unknown key.
                    map.next_value::<IgnoredAny>()?;

                    tracing::trace!("ran into an unknown key: {:?}", why);

                    continue;
                }
            };

            match key {
                GuildChannelField::Bitrate => {
                    if bitrate.is_some() {
                        return Err(DeError::duplicate_field("bitrate"));
                    }

                    bitrate = Some(map.next_value()?);
                }
                GuildChannelField::DefaultAutoArchiveDuration => {
                    if default_auto_archive_duration.is_some() {
                        return Err(DeError::duplicate_field("default_auto_archive_duration"));
                    }

                    default_auto_archive_duration = Some(map.next_value()?);
                }
                GuildChannelField::GuildId => {
                    if guild_id.is_some() {
                        return Err(DeError::duplicate_field("guild_id"));
                    }

                    guild_id = Some(map.next_value()?);
                }
                GuildChannelField::Id => {
                    if id.is_some() {
                        return Err(DeError::duplicate_field("id"));
                    }

                    id = Some(map.next_value()?);
                }
                GuildChannelField::Type => {
                    if kind.is_some() {
                        return Err(DeError::duplicate_field("type"));
                    }

                    kind = Some(map.next_value()?);
                }
                GuildChannelField::LastMessageId => {
                    if last_message_id.is_some() {
                        return Err(DeError::duplicate_field("last_message_id"));
                    }

                    last_message_id = Some(map.next_value()?);
                }
                GuildChannelField::LastPinTimestamp => {
                    if last_pin_timestamp.is_some() {
                        return Err(DeError::duplicate_field("last_pin_timestamp"));
                    }

                    last_pin_timestamp = Some(map.next_value()?);
                }
                GuildChannelField::Member => {
                    if member.is_some() {
                        return Err(DeError::duplicate_field("member"));
                    }

                    member = Some(map.next_value()?);
                }
                GuildChannelField::MemberCount => {
                    if member_count.is_some() {
                        return Err(DeError::duplicate_field("member_count"));
                    }

                    member_count = Some(map.next_value()?);
                }
                GuildChannelField::MessageCount => {
                    if message_count.is_some() {
                        return Err(DeError::duplicate_field("message_count"));
                    }

                    message_count = Some(map.next_value()?);
                }
                GuildChannelField::Name => {
                    if name.is_some() {
                        return Err(DeError::duplicate_field("name"));
                    }

                    name = Some(map.next_value()?);
                }
                GuildChannelField::Nsfw => {
                    if nsfw.is_some() {
                        return Err(DeError::duplicate_field("nsfw"));
                    }

                    nsfw = Some(map.next_value()?);
                }
                GuildChannelField::OwnerId => {
                    if owner_id.is_some() {
                        return Err(DeError::duplicate_field("owner_id"));
                    }

                    owner_id = Some(map.next_value()?);
                }
                GuildChannelField::ParentId => {
                    if parent_id.is_some() {
                        return Err(DeError::duplicate_field("parent_id"));
                    }

                    parent_id = Some(map.next_value()?);
                }
                GuildChannelField::PermissionOverwrites => {
                    if permission_overwrites.is_some() {
                        return Err(DeError::duplicate_field("permission_overwrites"));
                    }

                    permission_overwrites = Some(map.next_value()?);
                }
                GuildChannelField::Position => {
                    if position.is_some() {
                        return Err(DeError::duplicate_field("position"));
                    }

                    position = Some(map.next_value()?);
                }
                GuildChannelField::RateLimitPerUser => {
                    if rate_limit_per_user.is_some() {
                        return Err(DeError::duplicate_field("rate_limit_per_user"));
                    }

                    rate_limit_per_user = map.next_value::<Option<u64>>()?;
                }
                GuildChannelField::ThreadMetadata => {
                    if thread_metadata.is_some() {
                        return Err(DeError::duplicate_field("thread_metadata"));
                    }

                    thread_metadata = Some(map.next_value()?);
                }
                GuildChannelField::Topic => {
                    if topic.is_some() {
                        return Err(DeError::duplicate_field("topic"));
                    }

                    topic = Some(map.next_value()?);
                }
                GuildChannelField::UserLimit => {
                    if user_limit.is_some() {
                        return Err(DeError::duplicate_field("user_limit"));
                    }

                    user_limit = Some(map.next_value()?);
                }
                GuildChannelField::VideoQualityMode => {
                    if video_quality_mode.is_some() {
                        return Err(DeError::duplicate_field("video_quality_mode"));
                    }

                    video_quality_mode = Some(map.next_value()?);
                }
            }
        }

        // Now let's get all of the fields required by every guild channel
        // type.
        let id = id.ok_or_else(|| DeError::missing_field("id"))?;
        let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;
        let name = name.ok_or_else(|| DeError::missing_field("name"))?;

        let nsfw = nsfw.unwrap_or_default();
        let parent_id = parent_id.unwrap_or_default();

        tracing::trace!(
            %id,
            ?kind,
            %name,
            %nsfw,
            ?parent_id,
            "common fields of all variants exist"
        );

        Ok(match kind {
            ChannelType::GuildCategory => {
                let permission_overwrites = permission_overwrites
                    .ok_or_else(|| DeError::missing_field("permission_overwrites"))?;
                let position = position.ok_or_else(|| DeError::missing_field("position"))?;

                tracing::trace!(?permission_overwrites, %position, "handling category channel");

                GuildChannel::Category(CategoryChannel {
                    guild_id,
                    id,
                    kind,
                    name,
                    permission_overwrites,
                    position,
                })
            }
            ChannelType::GuildVoice | ChannelType::GuildStageVoice => {
                let bitrate = bitrate.ok_or_else(|| DeError::missing_field("bitrate"))?;
                let permission_overwrites = permission_overwrites
                    .ok_or_else(|| DeError::missing_field("permission_overwrites"))?;
                let position = position.ok_or_else(|| DeError::missing_field("position"))?;
                let user_limit = user_limit.ok_or_else(|| DeError::missing_field("user_limit"))?;

                tracing::trace!(
                    %bitrate,
                    ?permission_overwrites,
                    %position,
                    ?user_limit,
                    "handling voice channel"
                );

                let voice_channel = VoiceChannel {
                    id,
                    bitrate,
                    guild_id,
                    kind,
                    name,
                    parent_id,
                    permission_overwrites,
                    position,
                    rtc_region: None,
                    user_limit,
                    video_quality_mode,
                };

                if kind == ChannelType::GuildVoice {
                    GuildChannel::Voice(voice_channel)
                } else {
                    GuildChannel::Stage(voice_channel)
                }
            }
            ChannelType::GuildNews | ChannelType::GuildStore | ChannelType::GuildText => {
                let last_message_id = last_message_id.unwrap_or_default();
                let last_pin_timestamp = last_pin_timestamp.unwrap_or_default();
                let permission_overwrites = permission_overwrites
                    .ok_or_else(|| DeError::missing_field("permission_overwrites"))?;
                let position = position.ok_or_else(|| DeError::missing_field("position"))?;
                let topic = topic.unwrap_or_default();

                tracing::trace!(
                    ?last_message_id,
                    ?last_pin_timestamp,
                    ?permission_overwrites,
                    %position,
                    ?topic,
                    "handling news, store, or text channel"
                );

                GuildChannel::Text(TextChannel {
                    guild_id,
                    id,
                    kind,
                    last_message_id,
                    last_pin_timestamp,
                    name,
                    nsfw,
                    parent_id,
                    permission_overwrites,
                    position,
                    rate_limit_per_user,
                    topic,
                })
            }
            ChannelType::GuildNewsThread
            | ChannelType::GuildPrivateThread
            | ChannelType::GuildPublicThread => {
                let last_message_id = last_message_id.unwrap_or_default();
                let member = member.unwrap_or_default();
                let member_count = member_count.unwrap_or_default();
                let message_count = message_count.unwrap_or_default();
                let owner_id = owner_id.unwrap_or_default();
                let thread_metadata =
                    thread_metadata.ok_or_else(|| DeError::missing_field("thread_metadata"))?;

                match kind {
                    ChannelType::GuildNewsThread => {
                        tracing::trace!(
                            ?last_message_id,
                            ?member,
                            ?member_count,
                            ?message_count,
                            ?owner_id,
                            ?thread_metadata,
                            "handling news thread"
                        );

                        GuildChannel::NewsThread(NewsThread {
                            guild_id,
                            id,
                            kind,
                            last_message_id,
                            member,
                            member_count,
                            message_count,
                            name,
                            owner_id,
                            parent_id,
                            rate_limit_per_user,
                            thread_metadata,
                        })
                    }
                    ChannelType::GuildPrivateThread => {
                        let permission_overwrites = permission_overwrites.unwrap_or_default();

                        tracing::trace!(
                            ?last_message_id,
                            ?member,
                            ?member_count,
                            ?message_count,
                            ?owner_id,
                            ?permission_overwrites,
                            ?thread_metadata,
                            "handling private thread"
                        );

                        GuildChannel::PrivateThread(PrivateThread {
                            guild_id,
                            id,
                            kind,
                            last_message_id,
                            member,
                            member_count,
                            message_count,
                            name,
                            owner_id,
                            parent_id,
                            permission_overwrites,
                            rate_limit_per_user,
                            thread_metadata,
                        })
                    }
                    ChannelType::GuildPublicThread => {
                        tracing::trace!(
                            ?last_message_id,
                            ?member,
                            ?member_count,
                            ?message_count,
                            ?owner_id,
                            ?thread_metadata,
                            "handling public thread"
                        );

                        GuildChannel::PublicThread(PublicThread {
                            guild_id,
                            id,
                            kind,
                            last_message_id,
                            member,
                            member_count,
                            message_count,
                            name,
                            owner_id,
                            parent_id,
                            rate_limit_per_user,
                            thread_metadata,
                        })
                    }
                    _ => unreachable!(),
                }
            }
            other => return Err(DeError::unknown_variant(other.name(), VARIANTS)),
        })
    }
}

impl<'de> Deserialize<'de> for GuildChannel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(GuildChannelVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AutoArchiveDuration, CategoryChannel, Channel, ChannelType, Group, GuildChannel,
        NewsThread, PrivateChannel, PrivateThread, PublicThread, TextChannel, ThreadMember,
        ThreadMetadata, VoiceChannel,
    };
    use crate::{
        channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
        guild::Permissions,
        id::{ChannelId, GuildId, MessageId, UserId},
    };

    fn group() -> Group {
        Group {
            application_id: None,
            icon: None,
            id: ChannelId(123),
            kind: ChannelType::Group,
            last_message_id: None,
            last_pin_timestamp: None,
            name: Some("a group".to_owned()),
            owner_id: UserId(456),
            recipients: Vec::new(),
        }
    }

    fn guild_category() -> CategoryChannel {
        CategoryChannel {
            guild_id: Some(GuildId(321)),
            id: ChannelId(123),
            kind: ChannelType::GuildCategory,
            name: "category".to_owned(),
            permission_overwrites: Vec::new(),
            position: 0,
        }
    }

    fn guild_text() -> TextChannel {
        TextChannel {
            guild_id: Some(GuildId(321)),
            id: ChannelId(456),
            kind: ChannelType::GuildText,
            last_message_id: None,
            last_pin_timestamp: None,
            name: "text".to_owned(),
            nsfw: false,
            permission_overwrites: Vec::new(),
            parent_id: None,
            position: 1,
            rate_limit_per_user: None,
            topic: None,
        }
    }

    fn guild_voice() -> VoiceChannel {
        VoiceChannel {
            bitrate: 1000,
            guild_id: Some(GuildId(321)),
            id: ChannelId(789),
            kind: ChannelType::GuildVoice,
            name: "voice".to_owned(),
            permission_overwrites: Vec::new(),
            parent_id: None,
            position: 2,
            rtc_region: None,
            user_limit: None,
            video_quality_mode: None,
        }
    }

    fn guild_stage() -> VoiceChannel {
        VoiceChannel {
            bitrate: 1000,
            guild_id: Some(GuildId(321)),
            id: ChannelId(789),
            kind: ChannelType::GuildStageVoice,
            name: "stage".to_owned(),
            permission_overwrites: Vec::new(),
            parent_id: None,
            position: 2,
            rtc_region: None,
            user_limit: None,
            video_quality_mode: None,
        }
    }

    const fn private() -> PrivateChannel {
        PrivateChannel {
            id: ChannelId(234),
            last_message_id: None,
            last_pin_timestamp: None,
            kind: ChannelType::Private,
            recipients: Vec::new(),
        }
    }

    #[test]
    fn test_channel_helpers() {
        assert_eq!(Channel::Group(group()).id(), ChannelId(123));
        assert_eq!(
            Channel::Guild(GuildChannel::Category(guild_category())).id(),
            ChannelId(123)
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Text(guild_text())).id(),
            ChannelId(456)
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Voice(guild_voice())).id(),
            ChannelId(789)
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Stage(guild_stage())).id(),
            ChannelId(789)
        );
        assert_eq!(Channel::Private(private()).id(), ChannelId(234));
    }

    #[test]
    fn test_channel_name() {
        assert_eq!(Channel::Group(group()).name(), Some("a group"));
        let mut group_no_name = group();
        group_no_name.name = None;
        assert!(Channel::Group(group_no_name).name().is_none());
        assert_eq!(
            Channel::Guild(GuildChannel::Category(guild_category())).name(),
            Some("category")
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Text(guild_text())).name(),
            Some("text")
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Voice(guild_voice())).name(),
            Some("voice")
        );
        assert_eq!(
            Channel::Guild(GuildChannel::Stage(guild_stage())).name(),
            Some("stage")
        );
        assert!(Channel::Private(private()).name().is_none());
    }

    #[test]
    fn test_guild_channel_guild_id() {
        assert_eq!(
            GuildChannel::Category(guild_category()).guild_id(),
            Some(GuildId(321))
        );
        assert_eq!(
            GuildChannel::Text(guild_text()).guild_id(),
            Some(GuildId(321))
        );
        assert_eq!(
            GuildChannel::Voice(guild_voice()).guild_id(),
            Some(GuildId(321))
        );
        assert_eq!(
            GuildChannel::Stage(guild_stage()).guild_id(),
            Some(GuildId(321))
        );
    }

    #[test]
    fn test_guild_channel_id() {
        assert_eq!(
            GuildChannel::Category(guild_category()).id(),
            ChannelId(123)
        );
        assert_eq!(GuildChannel::Text(guild_text()).id(), ChannelId(456));
        assert_eq!(GuildChannel::Voice(guild_voice()).id(), ChannelId(789));
        assert_eq!(GuildChannel::Stage(guild_stage()).id(), ChannelId(789));
    }

    #[test]
    fn test_guild_channel_name() {
        assert_eq!(GuildChannel::Category(guild_category()).name(), "category");
        assert_eq!(GuildChannel::Text(guild_text()).name(), "text");
        assert_eq!(GuildChannel::Voice(guild_voice()).name(), "voice");
        assert_eq!(GuildChannel::Stage(guild_stage()).name(), "stage");
    }

    // The deserializer for GuildChannel should skip over fields names that
    // it couldn't deserialize.
    #[test]
    fn test_guild_channel_unknown_field_deserialization() {
        let input = serde_json::json!({
            "type": 0,
            "topic": "a",
            "rate_limit_per_user": 0,
            "position": 0,
            "permission_overwrites": [],
            "parent_id": null,
            "nsfw": false,
            "name": "hey",
            "last_message_id": "3",
            "id": "2",
            "guild_id": "1",
            "guild_hashes": {
                "version": 1,
                "roles": {
                    "hash": "aaaaaaaaaaa"
                },
                "metadata": {
                    "hash": "bbbbbbbbbbb"
                },
                "channels": {
                    "hash": "ccccccccccc"
                }
            },
            "unknown_field": "the deserializer should skip unknown field names",
        });

        let value = GuildChannel::Text(TextChannel {
            guild_id: Some(GuildId(1)),
            id: ChannelId(2),
            kind: ChannelType::GuildText,
            last_message_id: Some(MessageId(3)),
            last_pin_timestamp: None,
            name: "hey".to_owned(),
            nsfw: false,
            parent_id: None,
            permission_overwrites: Vec::new(),
            position: 0,
            rate_limit_per_user: Some(0),
            topic: Some("a".to_owned()),
        });

        assert_eq!(value, serde_json::from_value(input).unwrap());
    }

    #[test]
    fn test_guild_category_channel_deserialization() {
        let value = GuildChannel::Category(CategoryChannel {
            id: ChannelId(1),
            guild_id: Some(GuildId(2)),
            kind: ChannelType::GuildCategory,
            name: "foo".to_owned(),
            permission_overwrites: Vec::new(),
            position: 3,
        });
        let permission_overwrites: Vec<PermissionOverwrite> = Vec::new();

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "1",
                "guild_id": Some("2"),
                "name": "foo",
                "nsfw": false,
                "parent_id": None::<ChannelId>,
                "permission_overwrites": permission_overwrites,
                "position": 3,
                "type": 4,
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_guild_news_channel_deserialization() {
        let value = GuildChannel::Text(TextChannel {
            id: ChannelId(1),
            guild_id: Some(GuildId(2)),
            kind: ChannelType::GuildNews,
            last_message_id: Some(MessageId(4)),
            last_pin_timestamp: None,
            name: "news".to_owned(),
            nsfw: true,
            permission_overwrites: Vec::new(),
            parent_id: Some(ChannelId(5)),
            position: 3,
            rate_limit_per_user: None,
            topic: Some("a news channel".to_owned()),
        });
        let permission_overwrites: Vec<PermissionOverwrite> = Vec::new();

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "1",
                "guild_id": "2",
                "name": "news",
                "nsfw": true,
                "last_message_id": "4",
                "parent_id": "5",
                "permission_overwrites": permission_overwrites,
                "position": 3,
                "topic": "a news channel",
                "type": ChannelType::GuildNews,
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_guild_store_channel_deserialization() {
        let value = GuildChannel::Text(TextChannel {
            id: ChannelId(1),
            guild_id: Some(GuildId(2)),
            kind: ChannelType::GuildStore,
            last_message_id: None,
            last_pin_timestamp: None,
            name: "store".to_owned(),
            nsfw: false,
            permission_overwrites: Vec::new(),
            parent_id: None,
            position: 2,
            rate_limit_per_user: None,
            topic: Some("a store channel".to_owned()),
        });
        let permission_overwrites: Vec<PermissionOverwrite> = Vec::new();

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "1",
                "guild_id": "2",
                "name": "store",
                "nsfw": false,
                "permission_overwrites": permission_overwrites,
                "position": 2,
                "topic": "a store channel",
                "type": ChannelType::GuildStore,
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_guild_news_thread_deserialization() {
        let value = GuildChannel::NewsThread(NewsThread {
            guild_id: Some(GuildId(1)),
            id: ChannelId(6),
            kind: ChannelType::GuildNewsThread,
            last_message_id: Some(MessageId(3)),
            member: Some(ThreadMember {
                flags: 0_u64,
                id: Some(ChannelId(4)),
                join_timestamp: "jointimestamp".into(),
                user_id: Some(UserId(5)),
            }),
            member_count: 50_u8,
            message_count: 50_u8,
            name: "newsthread".into(),
            owner_id: Some(UserId(5)),
            parent_id: Some(ChannelId(2)),
            rate_limit_per_user: Some(1000_u64),
            thread_metadata: ThreadMetadata {
                archived: false,
                auto_archive_duration: AutoArchiveDuration::Day,
                archive_timestamp: "archivetimestamp".into(),
                locked: false,
            },
        });

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "6",
                "guild_id": "1",
                "type": ChannelType::GuildNewsThread,
                "last_message_id": "3",
                "member": {
                    "flags": 0,
                    "id": "4",
                    "join_timestamp": "jointimestamp",
                    "user_id": "5",
                },
                "member_count": 50,
                "message_count": 50,
                "name": "newsthread",
                "owner_id": "5",
                "parent_id": "2",
                "rate_limit_per_user": 1000,
                "thread_metadata": {
                    "archive_timestamp": "archivetimestamp",
                    "archived": false,
                    "auto_archive_duration": AutoArchiveDuration::Day,
                    "locked": false
                }
            }))
            .unwrap()
        )
    }

    #[test]
    fn test_guild_public_thread_deserialization() {
        let value = GuildChannel::PublicThread(PublicThread {
            guild_id: Some(GuildId(1)),
            id: ChannelId(6),
            kind: ChannelType::GuildPublicThread,
            last_message_id: Some(MessageId(3)),
            member: Some(ThreadMember {
                flags: 0_u64,
                id: Some(ChannelId(4)),
                join_timestamp: "jointimestamp".into(),
                user_id: Some(UserId(5)),
            }),
            member_count: 50_u8,
            message_count: 50_u8,
            name: "publicthread".into(),
            owner_id: Some(UserId(5)),
            parent_id: Some(ChannelId(2)),
            rate_limit_per_user: Some(1000_u64),
            thread_metadata: ThreadMetadata {
                archived: false,
                auto_archive_duration: AutoArchiveDuration::Day,
                archive_timestamp: "archivetimestamp".into(),
                locked: false,
            },
        });

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "6",
                "guild_id": "1",
                "type": ChannelType::GuildPublicThread,
                "last_message_id": "3",
                "member": {
                    "flags": 0,
                    "id": "4",
                    "join_timestamp": "jointimestamp",
                    "user_id": "5",
                },
                "member_count": 50,
                "message_count": 50,
                "name": "publicthread",
                "owner_id": "5",
                "parent_id": "2",
                "rate_limit_per_user": 1000,
                "thread_metadata": {
                    "archive_timestamp": "archivetimestamp",
                    "archived": false,
                    "auto_archive_duration": AutoArchiveDuration::Day,
                    "locked": false
                }
            }))
            .unwrap()
        )
    }

    #[test]
    fn test_guild_private_thread_deserialization() {
        let value = GuildChannel::PrivateThread(PrivateThread {
            guild_id: Some(GuildId(1)),
            id: ChannelId(6),
            kind: ChannelType::GuildPrivateThread,
            last_message_id: Some(MessageId(3)),
            member: Some(ThreadMember {
                flags: 0_u64,
                id: Some(ChannelId(4)),
                join_timestamp: "jointimestamp".into(),
                user_id: Some(UserId(5)),
            }),
            member_count: 50_u8,
            message_count: 50_u8,
            name: "privatethread".into(),
            owner_id: Some(UserId(5)),
            parent_id: Some(ChannelId(2)),
            rate_limit_per_user: Some(1000_u64),
            thread_metadata: ThreadMetadata {
                archived: false,
                auto_archive_duration: AutoArchiveDuration::Day,
                archive_timestamp: "archivetimestamp".into(),
                locked: false,
            },
            permission_overwrites: Vec::from([PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(UserId(5)),
            }]),
        });

        assert_eq!(
            value,
            serde_json::from_value(serde_json::json!({
                "id": "6",
                "guild_id": "1",
                "type": ChannelType::GuildPrivateThread,
                "last_message_id": "3",
                "member": {
                    "flags": 0,
                    "id": "4",
                    "join_timestamp": "jointimestamp",
                    "user_id": "5",
                },
                "member_count": 50,
                "message_count": 50,
                "name": "privatethread",
                "owner_id": "5",
                "parent_id": "2",
                "rate_limit_per_user": 1000,
                "thread_metadata": {
                    "archive_timestamp": "archivetimestamp",
                    "archived": false,
                    "auto_archive_duration": AutoArchiveDuration::Day,
                    "locked": false
                },
                "permission_overwrites": [
                    {
                        "allow": "0",
                        "deny": "0",
                        "type": 1,
                        "id": "5"
                    }
                ]
            }))
            .unwrap()
        )
    }
}
