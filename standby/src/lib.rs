//! # twilight-standby
//!
//! Standby is a utility to wait for an event to happen based on a predicate
//! check. For example, you may have a command that has a reaction menu of ✅ and
//! ❌. If you want to handle a reaction to these, using something like an
//! application-level state or event stream may not suit your use case. It may
//! be cleaner to wait for a reaction inline to your function. This is where
//! Twilight Standby comes in.
//!
//! Standby allows you to wait for things like an event in a certain guild
//! ([`Standby::wait_for`]), a new message in a channel
//! ([`Standby::wait_for_message`]), a new reaction on a message
//! ([`Standby::wait_for_reaction`]), and any event that might not take place in
//! a guild, such as a new `Ready` event ([`Standby::wait_for_event`]).
//!
//! To use Standby, you must process events with it in your main event loop.
//! Check out the [`Standby::process`] method.
//!
//! # Examples
//!
//! Wait for a message in channel 123 by user 456 with the content "test":
//!
//! ```no_run
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use futures_util::future;
//! use twilight_model::{gateway::payload::MessageCreate, id::{ChannelId, UserId}};
//! use twilight_standby::Standby;
//!
//! let standby = Standby::new();
//!
//! let message = standby.wait_for_message(ChannelId(123), |event: &MessageCreate| {
//!     event.author.id == UserId(456) && event.content == "test"
//! }).await;
//! # Ok(()) }
//! ```
//!
//! For more examples, check out each method.
//!
//! [`Standby::process`]: struct.Standby.html#method.process
//! [`Standby::wait_for`]: struct.Standby.html#method.wait_for
//! [`Standby::wait_for_event`]: struct.Standby.html#method.wait_for_event
//! [`Standby::wait_for_message`]: struct.Standby.html#method.wait_for_message
//! [`Standby::wait_for_reaction`]: struct.Standby.html#method.wait_for_reaction

use futures_channel::mpsc::{self, Sender};
use futures_util::{lock::Mutex, stream::StreamExt};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::Arc,
};
use twilight_model::{
    channel::{Channel, GuildChannel},
    gateway::payload::{Event, EventType, MessageCreate, ReactionAdd},
    id::{ChannelId, GuildId, MessageId},
};

struct Bystander<E> {
    func: Box<dyn Fn(&E) -> bool>,
    sender: Sender<E>,
}

impl<E> Debug for Bystander<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Bystander")
            .field("check", &"check func")
            .field("sender", &"mpsc sender")
            .finish()
    }
}

#[derive(Debug, Default)]
struct StandbyRef {
    events: Mutex<HashMap<EventType, Vec<Bystander<Event>>>>,
    guilds: Mutex<HashMap<GuildId, Vec<Bystander<Event>>>>,
    messages: Mutex<HashMap<ChannelId, Vec<Bystander<MessageCreate>>>>,
    reactions: Mutex<HashMap<MessageId, Vec<Bystander<ReactionAdd>>>>,
}

/// The `Standby` struct, used by the main event loop to process events and by
/// tasks to wait for an event.
///
/// Refer to the crate-level documentation for more information.
#[derive(Clone, Debug, Default)]
pub struct Standby(Arc<StandbyRef>);

impl Standby {
    /// Create a new instance of `Standby`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process an event, calling any bystanders that might be waiting on it.
    ///
    /// When a bystander checks to see if an event is what it's waiting for, it
    /// will receive the event by cloning it.
    pub async fn process(&self, event: &Event) {
        match event {
            Event::MessageCreate(e) => return self.process_message(e.0.channel_id, &e).await,
            Event::ReactionAdd(e) => return self.process_reaction(e.0.message_id, &e).await,
            _ => {}
        }

        match event_guild_id(event) {
            Some(guild_id) => self.process_guild(guild_id, event).await,
            None => self.process_event(event).await,
        }
    }

    /// Wait for an event in a certain guild.
    ///
    /// Returns `None` if the `Standby` instance was dropped or this waiter was
    /// dropped before an event could be found.
    ///
    /// # Examples
    ///
    /// Wait for a `BanAdd` event in guild 123:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::{
    ///     gateway::payload::{EventType, Event},
    ///     id::GuildId,
    /// };
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let reaction = standby.wait_for(GuildId(123), |event: &Event| {
    ///     event.kind() == EventType::BanAdd
    /// }).await;
    /// # Ok(()) }
    /// ```
    ///
    /// [`Standby`]: struct.Standby.html
    pub async fn wait_for<F: Fn(&Event) -> bool + 'static>(
        &self,
        guild_id: GuildId,
        check: impl Into<Box<F>>,
    ) -> Option<Event> {
        let (tx, mut rx) = mpsc::channel(1);

        {
            let mut guilds = self.0.guilds.lock().await;

            let guild = guilds.entry(guild_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: tx,
            });
        }

        rx.next().await
    }

    /// Wait for an event not in a certain guild. This must be filtered by an
    /// event type.
    ///
    /// Returns `None` if the `Standby` instance was dropped or this waiter was
    /// dropped before an event could be found.
    ///
    /// # Examples
    ///
    /// Wait for a `Ready` event for shard 5:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::{gateway::payload::{EventType, Event}};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let ready = standby.wait_for_event(EventType::Ready, |event: &Event| {
    ///     if let Event::Ready(ready) = event {
    ///         ready.shard.map(|[id, _]| id == 5).unwrap_or(false)
    ///     } else {
    ///         false
    ///     }
    /// }).await;
    /// # Ok(()) }
    /// ```
    ///
    /// [`Standby`]: struct.Standby.html
    pub async fn wait_for_event<F: Fn(&Event) -> bool + 'static>(
        &self,
        event_type: EventType,
        check: impl Into<Box<F>>,
    ) -> Option<Event> {
        let (tx, mut rx) = mpsc::channel(1);

        {
            let mut events = self.0.events.lock().await;

            let guild = events.entry(event_type).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: tx,
            });
        }

        rx.next().await
    }

    /// Wait for a message in a certain channel.
    ///
    /// Returns `None` if the `Standby` instance was dropped or this waiter was
    /// dropped before an event could be found.
    ///
    /// # Examples
    ///
    /// Wait for a message in channel 123 by user 456 with the content "test":
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::{gateway::payload::MessageCreate, id::{ChannelId, UserId}};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let message = standby.wait_for_message(ChannelId(123), |event: &MessageCreate| {
    ///     event.author.id == UserId(456) && event.content == "test"
    /// }).await;
    /// # Ok(()) }
    /// ```
    ///
    /// [`Standby`]: struct.Standby.html
    pub async fn wait_for_message<F: Fn(&MessageCreate) -> bool + 'static>(
        &self,
        channel_id: ChannelId,
        check: impl Into<Box<F>>,
    ) -> Option<MessageCreate> {
        let (tx, mut rx) = mpsc::channel(1);

        {
            let mut messages = self.0.messages.lock().await;

            let guild = messages.entry(channel_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: tx,
            });
        }

        rx.next().await
    }

    /// Wait for a reaction on a certain message.
    ///
    /// Returns `None` if the `Standby` instance was dropped or this waiter was
    /// dropped before an event could be found.
    ///
    /// # Examples
    ///
    /// Wait for a reaction on message 123 by user 456:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::{gateway::payload::ReactionAdd, id::{MessageId, UserId}};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let reaction = standby.wait_for_reaction(MessageId(123), |event: &ReactionAdd| {
    ///     event.user_id == UserId(456)
    /// }).await;
    /// # Ok(()) }
    /// ```
    ///
    /// [`Standby`]: struct.Standby.html
    pub async fn wait_for_reaction<F: Fn(&ReactionAdd) -> bool + 'static>(
        &self,
        message_id: MessageId,
        check: impl Into<Box<F>>,
    ) -> Option<ReactionAdd> {
        let (tx, mut rx) = mpsc::channel(1);

        {
            let mut reactions = self.0.reactions.lock().await;

            let guild = reactions.entry(message_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: tx,
            });
        }

        rx.next().await
    }

    async fn process_event(&self, event: &Event) {
        let mut events = self.0.events.lock().await;

        if let Some(mut bystanders) = events.get_mut(&event.kind()) {
            self.iter_bystanders(&mut bystanders, event);
        }
    }

    async fn process_guild(&self, guild_id: GuildId, event: &Event) {
        let mut guilds = self.0.guilds.lock().await;

        if let Some(mut bystanders) = guilds.get_mut(&guild_id) {
            self.iter_bystanders(&mut bystanders, event);
        }
    }

    async fn process_message(&self, channel_id: ChannelId, event: &MessageCreate) {
        let mut messages = self.0.messages.lock().await;

        if let Some(mut bystanders) = messages.get_mut(&channel_id) {
            self.iter_bystanders(&mut bystanders, event);
        }
    }

    async fn process_reaction(&self, message_id: MessageId, event: &ReactionAdd) {
        let mut reactions = self.0.reactions.lock().await;

        if let Some(mut bystanders) = reactions.get_mut(&message_id) {
            self.iter_bystanders(&mut bystanders, event);
        }
    }

    fn iter_bystanders<E: Clone>(&self, bystanders: &mut Vec<Bystander<E>>, event: &E) {
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain_filter
        // https://github.com/rust-lang/rust/issues/43244
        let mut remove = Vec::new();

        for (idx, bystander) in bystanders.iter_mut().enumerate() {
            if bystander.sender.is_closed() {
                remove.push(idx);

                continue;
            }

            if !(bystander.func)(event) {
                continue;
            }

            let _ = bystander.sender.try_send(event.clone());

            remove.push(idx);
        }
    }
}

fn event_guild_id(event: &Event) -> Option<GuildId> {
    match event {
        Event::BanAdd(e) => Some(e.guild_id),
        Event::BanRemove(e) => Some(e.guild_id),
        Event::ChannelCreate(e) => channel_guild_id(e),
        Event::ChannelDelete(e) => channel_guild_id(e),
        Event::ChannelPinsUpdate(_) => None,
        Event::ChannelUpdate(e) => channel_guild_id(e),
        Event::GuildCreate(e) => Some(e.id),
        Event::GuildDelete(e) => Some(e.id),
        Event::GuildEmojisUpdate(e) => Some(e.guild_id),
        Event::GuildIntegrationsUpdate(e) => Some(e.guild_id),
        Event::GuildUpdate(e) => Some(e.id),
        Event::InviteCreate(e) => Some(e.guild_id),
        Event::InviteDelete(e) => Some(e.guild_id),
        Event::MemberAdd(e) => e.guild_id,
        Event::MemberChunk(e) => Some(e.guild_id),
        Event::MemberRemove(e) => Some(e.guild_id),
        Event::MemberUpdate(e) => Some(e.guild_id),
        Event::MessageCreate(e) => e.guild_id,
        Event::MessageDelete(_) => None,
        Event::MessageDeleteBulk(_) => None,
        Event::MessageUpdate(_) => None,
        Event::PresenceUpdate(e) => e.guild_id,
        Event::PresencesReplace => None,
        Event::ReactionAdd(e) => e.guild_id,
        Event::ReactionRemove(e) => e.guild_id,
        Event::ReactionRemoveAll(e) => e.guild_id,
        Event::ReactionRemoveEmoji(e) => Some(e.guild_id),
        Event::Ready(_) => None,
        Event::Resumed => None,
        Event::RoleCreate(e) => Some(e.guild_id),
        Event::RoleDelete(e) => Some(e.guild_id),
        Event::RoleUpdate(e) => Some(e.guild_id),
        Event::TypingStart(e) => e.guild_id,
        Event::UnavailableGuild(e) => Some(e.id),
        Event::UserUpdate(_) => None,
        Event::VoiceServerUpdate(e) => e.guild_id,
        Event::VoiceStateUpdate(e) => e.0.guild_id,
        Event::WebhooksUpdate(e) => Some(e.guild_id),
    }
}

fn channel_guild_id(channel: &Channel) -> Option<GuildId> {
    match channel {
        Channel::Guild(c) => match c {
            GuildChannel::Category(c) => c.guild_id,
            GuildChannel::Text(c) => c.guild_id,
            GuildChannel::Voice(c) => c.guild_id,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Standby;
    use futures_util::future;
    use std::collections::HashMap;
    use twilight_model::{
        channel::message::{Message, MessageType},
        gateway::payload::{Event, MessageCreate, RoleDelete},
        id::{ChannelId, GuildId, MessageId, RoleId, UserId},
        user::User,
    };

    #[tokio::test]
    async fn test_wait_for_simple() {
        let standby = Standby::new();
        let wait = standby.wait_for(GuildId(1), |event: &Event| match event {
            Event::RoleDelete(e) => e.guild_id == GuildId(1),
            _ => false,
        });
        let process = standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(2),
        }));

        // wait always gets polled first
        let (res, _) = future::join(wait, process).await;

        assert!(matches!(
            res,
            Some(Event::RoleDelete(RoleDelete {
                guild_id: GuildId(1),
                role_id: RoleId(2),
            }))
        ));
    }

    #[tokio::test]
    async fn test_wait_for_message() {
        let message = Message {
            id: MessageId(3),
            activity: None,
            application: None,
            attachments: Vec::new(),
            author: User {
                avatar: None,
                bot: false,
                discriminator: "0001".to_owned(),
                email: None,
                flags: None,
                id: UserId(2),
                locale: None,
                mfa_enabled: None,
                name: "twilight".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            },
            channel_id: ChannelId(1),
            content: "test".to_owned(),
            edited_timestamp: None,
            embeds: Vec::new(),
            flags: None,
            guild_id: Some(GuildId(4)),
            kind: MessageType::Regular,
            member: None,
            mention_channels: Vec::new(),
            mention_everyone: false,
            mention_roles: Vec::new(),
            mentions: HashMap::new(),
            pinned: false,
            reactions: Vec::new(),
            reference: None,
            timestamp: String::new(),
            tts: false,
            webhook_id: None,
        };
        let event = Event::MessageCreate(Box::new(MessageCreate(message)));

        let standby = Standby::new();
        let wait = standby.wait_for_message(ChannelId(1), |message: &MessageCreate| {
            message.author.id == UserId(2)
        });

        let process = standby.process(&event);

        // wait always gets polled first
        let (res, _) = future::join(wait, process).await;

        assert_eq!(Some(MessageId(3)), res.map(|msg| msg.id));
    }
}
