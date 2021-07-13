use crate::{config::ResourceType, model::CachedMessage, InMemoryCache, UpdateCache};
use std::borrow::Cow;
use twilight_model::gateway::payload::{
    MessageCreate, MessageDelete, MessageDeleteBulk, MessageUpdate,
};

impl UpdateCache for MessageCreate {
    fn update(&self, cache: &InMemoryCache) {
        if cache.wants(ResourceType::USER) {
            cache.cache_user(Cow::Borrowed(&self.author), self.guild_id);
        }

        if let (Some(member), Some(guild_id), true) = (
            &self.member,
            self.guild_id,
            cache.wants(ResourceType::MEMBER),
        ) {
            cache.cache_borrowed_partial_member(guild_id, member, self.author.id)
        }

        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel = cache.0.messages.entry(self.0.channel_id).or_default();

        if channel.len() > cache.0.config.message_cache_size() {
            channel.pop_back();
        }

        channel.push_front(CachedMessage::from(self.0.clone()));
    }
}

impl UpdateCache for MessageDelete {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel = cache.0.messages.entry(self.channel_id).or_default();

        if let Some(idx) = channel.iter().position(|msg| msg.id == self.id) {
            channel.remove(idx);
        }
    }
}

impl UpdateCache for MessageDeleteBulk {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel = cache.0.messages.entry(self.channel_id).or_default();

        for id in &self.ids {
            if let Some(idx) = channel.iter().position(|msg| &msg.id == id) {
                channel.remove(idx);
            }
        }
    }
}

impl UpdateCache for MessageUpdate {
    fn update(&self, cache: &InMemoryCache) {
        if !cache.wants(ResourceType::MESSAGE) {
            return;
        }

        let mut channel = cache.0.messages.entry(self.channel_id).or_default();

        if let Some(mut message) = channel.iter_mut().find(|msg| msg.id == self.id) {
            if let Some(attachments) = &self.attachments {
                message.attachments = attachments.clone();
            }

            if let Some(content) = &self.content {
                message.content = content.clone();
            }

            if let Some(edited_timestamp) = &self.edited_timestamp {
                message.edited_timestamp.replace(edited_timestamp.clone());
            }

            if let Some(embeds) = &self.embeds {
                message.embeds = embeds.clone();
            }

            if let Some(mention_everyone) = self.mention_everyone {
                message.mention_everyone = mention_everyone;
            }

            if let Some(mention_roles) = &self.mention_roles {
                message.mention_roles = mention_roles.clone();
            }

            if let Some(mentions) = &self.mentions {
                message.mentions = mentions.iter().map(|x| x.id).collect::<Vec<_>>();
            }

            if let Some(pinned) = self.pinned {
                message.pinned = pinned;
            }

            if let Some(timestamp) = &self.timestamp {
                message.timestamp = timestamp.clone();
            }

            if let Some(tts) = self.tts {
                message.tts = tts;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::{
        channel::message::{Message, MessageFlags, MessageType},
        guild::PartialMember,
        id::{ChannelId, GuildId, MessageId, UserId},
        user::User,
    };

    #[test]
    fn test_message_create() {
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE | ResourceType::MEMBER | ResourceType::USER)
            .message_cache_size(1)
            .build();
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

        {
            let entry = cache.0.users.get(&UserId(3)).unwrap();
            assert_eq!(entry.value().1.len(), 1);
        }
        assert_eq!(
            cache.member(GuildId(1), UserId(3)).unwrap().user_id,
            UserId(3),
        );
        {
            let entry = cache.0.messages.get(&ChannelId(2)).unwrap();
            assert_eq!(entry.value().len(), 1);
        }
    }
}
