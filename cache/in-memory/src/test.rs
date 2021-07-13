use crate::InMemoryCache;
use twilight_model::{
    channel::{
        message::{Message, MessageFlags, MessageType},
        ChannelType, GuildChannel, Reaction, ReactionType, TextChannel,
    },
    gateway::payload::{MessageCreate, ReactionAdd},
    guild::{Emoji, Member, PartialMember, Permissions, Role},
    id::{ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId},
    user::{CurrentUser, User},
    voice::VoiceState,
};

pub fn cache_with_message_and_reactions() -> InMemoryCache {
    let cache = InMemoryCache::new();

    let msg = Message {
        activity: None,
        application: None,
        application_id: None,
        attachments: Vec::new(),
        author: User {
            avatar: Some("".to_owned()),
            bot: false,
            discriminator: "0001".to_owned(),
            email: None,
            flags: None,
            id: UserId(3),
            locale: None,
            mfa_enabled: None,
            name: "test".to_owned(),
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None,
        },
        channel_id: ChannelId(2),
        components: Vec::new(),
        content: "ping".to_owned(),
        edited_timestamp: None,
        embeds: Vec::new(),
        flags: Some(MessageFlags::empty()),
        guild_id: Some(GuildId(1)),
        id: MessageId(4),
        interaction: None,
        kind: MessageType::Regular,
        member: Some(PartialMember {
            deaf: false,
            joined_at: None,
            mute: false,
            nick: Some("member nick".to_owned()),
            permissions: None,
            premium_since: None,
            roles: Vec::new(),
            user: None,
        }),
        mention_channels: Vec::new(),
        mention_everyone: false,
        mention_roles: Vec::new(),
        mentions: Vec::new(),
        pinned: false,
        reactions: Vec::new(),
        reference: None,
        stickers: Vec::new(),
        referenced_message: None,
        timestamp: String::new(),
        tts: false,
        webhook_id: None,
    };

    cache.update(&MessageCreate(msg));

    let mut reaction = ReactionAdd(Reaction {
        channel_id: ChannelId(2),
        emoji: ReactionType::Unicode {
            name: "😀".to_owned(),
        },
        guild_id: Some(GuildId(1)),
        member: Some(Member {
            deaf: false,
            guild_id: GuildId(1),
            hoisted_role: None,
            joined_at: None,
            mute: false,
            nick: Some("member nick".to_owned()),
            pending: false,
            premium_since: None,
            roles: Vec::new(),
            user: User {
                avatar: Some("".to_owned()),
                bot: false,
                discriminator: "0001".to_owned(),
                email: None,
                flags: None,
                id: UserId(3),
                locale: None,
                mfa_enabled: None,
                name: "test".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            },
        }),
        message_id: MessageId(4),
        user_id: UserId(3),
    });

    cache.update(&reaction);

    reaction.member.replace(Member {
        deaf: false,
        guild_id: GuildId(1),
        hoisted_role: None,
        joined_at: None,
        mute: false,
        nick: None,
        pending: false,
        premium_since: None,
        roles: Vec::new(),
        user: User {
            avatar: Some("".to_owned()),
            bot: false,
            discriminator: "0002".to_owned(),
            email: None,
            flags: None,
            id: UserId(5),
            locale: None,
            mfa_enabled: None,
            name: "test".to_owned(),
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None,
        },
    });
    reaction.user_id = UserId(5);

    cache.update(&reaction);

    reaction.emoji = ReactionType::Unicode {
        name: "🗺️".to_owned(),
    };

    cache.update(&reaction);

    cache
}

pub fn current_user(id: u64) -> CurrentUser {
    CurrentUser {
        avatar: None,
        bot: true,
        discriminator: "9876".to_owned(),
        email: None,
        id: UserId(id),
        mfa_enabled: true,
        name: "test".to_owned(),
        verified: Some(true),
        premium_type: None,
        public_flags: None,
        flags: None,
        locale: None,
    }
}

pub fn emoji(id: EmojiId, user: Option<User>) -> Emoji {
    Emoji {
        animated: false,
        available: true,
        id,
        managed: false,
        name: "test".to_owned(),
        require_colons: true,
        roles: Vec::new(),
        user,
    }
}

pub fn guild_channel_text() -> (GuildId, ChannelId, GuildChannel) {
    let guild_id = GuildId(1);
    let channel_id = ChannelId(2);
    let channel = GuildChannel::Text(TextChannel {
        guild_id: Some(guild_id),
        id: channel_id,
        kind: ChannelType::GuildText,
        last_message_id: None,
        last_pin_timestamp: None,
        name: "test".to_owned(),
        nsfw: false,
        parent_id: None,
        permission_overwrites: Vec::new(),
        position: 3,
        rate_limit_per_user: None,
        topic: None,
    });

    (guild_id, channel_id, channel)
}

pub fn member(id: UserId, guild_id: GuildId) -> Member {
    Member {
        deaf: false,
        guild_id,
        hoisted_role: None,
        joined_at: None,
        mute: false,
        nick: None,
        pending: false,
        premium_since: None,
        roles: Vec::new(),
        user: user(id),
    }
}

pub fn role(id: RoleId) -> Role {
    Role {
        color: 0,
        hoist: false,
        id,
        managed: false,
        mentionable: false,
        name: "test".to_owned(),
        permissions: Permissions::empty(),
        position: 0,
        tags: None,
    }
}

pub fn voice_state(
    guild_id: GuildId,
    channel_id: Option<ChannelId>,
    user_id: UserId,
) -> VoiceState {
    VoiceState {
        channel_id,
        deaf: false,
        guild_id: Some(guild_id),
        member: None,
        mute: true,
        self_deaf: false,
        self_mute: true,
        self_stream: false,
        session_id: "a".to_owned(),
        suppress: false,
        token: None,
        user_id,
        request_to_speak_timestamp: Some("2021-04-21T22:16:50+0000".to_owned()),
    }
}

pub fn user(id: UserId) -> User {
    User {
        avatar: None,
        bot: false,
        discriminator: "0001".to_owned(),
        email: None,
        flags: None,
        id,
        locale: None,
        mfa_enabled: None,
        name: "user".to_owned(),
        premium_type: None,
        public_flags: None,
        system: None,
        verified: None,
    }
}
