//! # twilight-standby
//!
//! [![discord badge][]][discord link] [![github badge][]][github link] [![license badge][]][license link] ![rust badge]
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
//! a guild, such as a new `Ready` event ([`Standby::wait_for_event`]). Each
//! method also has a stream variant.
//!
//! To use Standby, you must process events with it in your main event loop.
//! Check out the [`Standby::process`] method.
//!
//! ## When to use futures and streams
//!
//! `Standby` has two variants of each method: a future variant and a stream
//! variant. An example is [`Standby::wait_for_message`], which also has a
//! [`Standby::wait_for_message_stream`] variant. The future variant is useful
//! when you want to oneshot an event that you need to wait for. This means that
//! if you only need to wait for one message in a channel to come in, you'd use
//! the future variant. If you need to wait for multiple messages, such as maybe
//! all of the messages within a minute's timespan, you'd use the
//! [`Standby::wait_for_message_stream`] method.
//!
//! The difference is that if you use the futures variant in a loop then you may
//! miss some events while processing a received event. By using a stream, you
//! won't miss any events.
//!
//! ## Examples
//!
//! ### At a glance
//!
//! Wait for a message in channel 123 by user 456 with the content "test":
//!
//! ```rust,no_run
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use twilight_model::{gateway::payload::MessageCreate, id::{ChannelId, UserId}};
//! use twilight_standby::Standby;
//!
//! let standby = Standby::new();
//!
//! let message = standby.wait_for_message(ChannelId(123), |event: &MessageCreate| {
//!     event.author.id == UserId(456) && event.content == "test"
//! }).await?;
//! # Ok(()) }
//! ```
//!
//! ### A full example
//!
//! A full sample bot connecting to the gateway, processing events, and
//! including a handler to wait for reactions:
//!
//! ```rust,no_run
//! use futures_util::StreamExt;
//! use std::{env, error::Error};
//! use twilight_gateway::{Event, Intents, Shard};
//! use twilight_model::{
//!     channel::Message,
//!     gateway::payload::ReactionAdd,
//!     id::{ChannelId, UserId},
//! };
//! use twilight_standby::Standby;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Start a shard connected to the gateway to receive events.
//!     let intents = Intents::GUILD_MESSAGES | Intents::GUILD_MESSAGE_REACTIONS;
//!     let (shard, mut events) = Shard::new(env::var("DISCORD_TOKEN")?, intents);
//!     shard.start().await?;
//!
//!     let standby = Standby::new();
//!
//!     while let Some(event) = events.next().await {
//!         // Have standby process the event, which will fulfill any futures that
//!         // are waiting for an event.
//!         standby.process(&event);
//!
//!         match event {
//!             Event::MessageCreate(msg) if msg.content == "!react" => {
//!                 tokio::spawn(react(msg.0, standby.clone()));
//!             },
//!             _ => {},
//!         }
//!     }
//!
//!     Ok(())
//! }
//!
//! // Wait for a reaction from the user who sent the message, and then print it
//! // once they react.
//! async fn react(msg: Message, standby: Standby) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
//!     let author_id = msg.author.id;
//!
//!     let reaction = standby.wait_for_reaction(msg.id, move |event: &ReactionAdd| {
//!         event.user_id == author_id
//!     }).await?;
//!
//!     println!("user reacted with {:?}", reaction.emoji);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more examples, check out each of the methods on [`Standby`].
//!
//! [discord badge]: https://img.shields.io/discord/745809834183753828?color=%237289DA&label=discord%20server&logo=discord&style=for-the-badge
//! [discord link]: https://discord.gg/7jj8n7D
//! [github badge]: https://img.shields.io/badge/github-twilight-6f42c1.svg?style=for-the-badge&logo=github
//! [github link]: https://github.com/twilight-rs/twilight
//! [license badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=for-the-badge&logo=pastebin
//! [license link]: https://github.com/twilight-rs/twilight/blob/main/LICENSE.md
//! [rust badge]: https://img.shields.io/badge/rust-1.49+-93450a.svg?style=for-the-badge&logo=rust

#![deny(
    broken_intra_doc_links,
    clippy::missing_const_for_fn,
    missing_docs,
    rust_2018_idioms,
    unused,
    warnings
)]

mod futures;

pub use futures::{
    WaitForEventFuture, WaitForEventStream, WaitForGuildEventFuture, WaitForGuildEventStream,
    WaitForMessageFuture, WaitForMessageStream, WaitForReactionFuture, WaitForReactionStream,
};

use dashmap::DashMap;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{
    mpsc::{self, UnboundedSender as MpscSender},
    oneshot::{self, Sender as OneshotSender},
};
use twilight_model::{
    channel::Channel,
    gateway::{
        event::Event,
        payload::{MessageCreate, ReactionAdd},
    },
    id::{ChannelId, GuildId, MessageId},
};

enum Sender<E> {
    Mpsc(MpscSender<E>),
    Oneshot(OneshotSender<E>),
}

impl<E> Sender<E> {
    fn is_closed(&self) -> bool {
        match self {
            Self::Mpsc(sender) => sender.is_closed(),
            Self::Oneshot(sender) => sender.is_closed(),
        }
    }
}

struct Bystander<E> {
    func: Box<dyn Fn(&E) -> bool + Send + Sync>,
    sender: Option<Sender<E>>,
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
    events: DashMap<u64, Bystander<Event>>,
    event_counter: AtomicU64,
    guilds: DashMap<GuildId, Vec<Bystander<Event>>>,
    messages: DashMap<ChannelId, Vec<Bystander<MessageCreate>>>,
    reactions: DashMap<MessageId, Vec<Bystander<ReactionAdd>>>,
}

/// The `Standby` struct, used by the main event loop to process events and by
/// tasks to wait for an event.
///
/// Refer to the crate-level documentation for more information.
///
/// # Cloning
///
/// Standby internally wraps its data within an Arc. This means that standby can
/// be cloned and passed around tasks and threads cheaply.
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
    ///
    /// This function must be called when events are received in order for
    /// futures returned by methods to fulfill.
    pub fn process(&self, event: &Event) {
        tracing::trace!(event_type = ?event.kind(), ?event, "processing event");

        match event {
            Event::MessageCreate(e) => self.process_message(e.0.channel_id, &e),
            Event::ReactionAdd(e) => self.process_reaction(e.0.message_id, &e),
            _ => {}
        }

        if let Some(guild_id) = event_guild_id(event) {
            self.process_guild(guild_id, event);
        }

        self.process_event(event);
    }

    /// Wait for an event in a certain guild.
    ///
    /// Returns a Canceled error if the Standby struct was dropped.
    ///
    /// If you need to wait for multiple guild events matching the given
    /// predicate, use [`wait_for_stream`].
    ///
    /// # Examples
    ///
    /// Wait for a `BanAdd` event in guild 123:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::{
    ///     gateway::event::{EventType, Event},
    ///     id::GuildId,
    /// };
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let reaction = standby.wait_for(GuildId(123), |event: &Event| {
    ///     event.kind() == EventType::BanAdd
    /// }).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_stream`]: Self::wait_for_stream
    pub fn wait_for<F: Fn(&Event) -> bool + Send + Sync + 'static>(
        &self,
        guild_id: GuildId,
        check: impl Into<Box<F>>,
    ) -> WaitForGuildEventFuture {
        tracing::trace!(%guild_id, "waiting for event in guild");
        let (tx, rx) = oneshot::channel();

        {
            let mut guild = self.0.guilds.entry(guild_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Oneshot(tx)),
            });
        }

        WaitForGuildEventFuture { rx }
    }

    /// Wait for a stream of events in a certain guild.
    ///
    /// Returns a Canceled error if the Standby struct was dropped.
    ///
    /// If you need to wait for only one guild event matching the given
    /// predicate, use [`wait_for`].
    ///
    /// # Examples
    ///
    /// Wait for multiple `BanAdd` events in guild 123:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::stream::StreamExt;
    /// use twilight_model::{
    ///     gateway::event::{EventType, Event},
    ///     id::GuildId,
    /// };
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let mut stream = standby.wait_for_stream(GuildId(123), |event: &Event| {
    ///     event.kind() == EventType::BanAdd
    /// });
    ///
    /// while let Some(event) = stream.next().await {
    ///     if let Event::BanAdd(ban) = event {
    ///         println!("user {} was banned in guild {}", ban.user.id, ban.guild_id);
    ///     }
    ///  }
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for`]: Self::wait_for
    pub fn wait_for_stream<F: Fn(&Event) -> bool + Send + Sync + 'static>(
        &self,
        guild_id: GuildId,
        check: impl Into<Box<F>>,
    ) -> WaitForGuildEventStream {
        tracing::trace!(%guild_id, "waiting for event in guild");
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut guild = self.0.guilds.entry(guild_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Mpsc(tx)),
            });
        }

        WaitForGuildEventStream { rx }
    }

    /// Wait for an event not in a certain guild. This must be filtered by an
    /// event type.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for multiple events matching the given predicate,
    /// use [`wait_for_event_stream`].
    ///
    /// # Examples
    ///
    /// Wait for a `Ready` event for shard 5:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::future;
    /// use twilight_model::gateway::event::{EventType, Event};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let ready = standby.wait_for_event(|event: &Event| {
    ///     if let Event::Ready(ready) = event {
    ///         ready.shard.map(|[id, _]| id == 5).unwrap_or(false)
    ///     } else {
    ///         false
    ///     }
    /// }).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_event_stream`]: Self::wait_for_event_stream
    pub fn wait_for_event<F: Fn(&Event) -> bool + Send + Sync + 'static>(
        &self,
        check: impl Into<Box<F>>,
    ) -> WaitForEventFuture {
        tracing::trace!("waiting for event");
        let (tx, rx) = oneshot::channel();

        {
            self.0.events.insert(
                self.next_event_id(),
                Bystander {
                    func: check.into(),
                    sender: Some(Sender::Oneshot(tx)),
                },
            );
        }

        WaitForEventFuture { rx }
    }

    /// Wait for a stream of events not in a certain guild. This must be
    /// filtered by an event type.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for only one event matching the given predicate, use
    /// [`wait_for_event`].
    ///
    /// # Examples
    ///
    /// Wait for multiple `Ready` events on shard 5:
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::stream::StreamExt;
    /// use twilight_model::gateway::event::{EventType, Event};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let mut events = standby.wait_for_event_stream(|event: &Event| {
    ///     if let Event::Ready(ready) = event {
    ///         ready.shard.map(|[id, _]| id == 5).unwrap_or(false)
    ///     } else {
    ///         false
    ///     }
    /// });
    ///
    /// while let Some(event) = events.next().await {
    ///     println!("got event with type {:?}", event.kind());
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_event`]: Self::wait_for_event
    pub fn wait_for_event_stream<F: Fn(&Event) -> bool + Send + Sync + 'static>(
        &self,
        check: impl Into<Box<F>>,
    ) -> WaitForEventStream {
        tracing::trace!("waiting for event");
        let (tx, rx) = mpsc::unbounded_channel();

        {
            self.0.events.insert(
                self.next_event_id(),
                Bystander {
                    func: check.into(),
                    sender: Some(Sender::Mpsc(tx)),
                },
            );
        }

        WaitForEventStream { rx }
    }

    /// Wait for a message in a certain channel.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for multiple messages matching the given predicate,
    /// use [`wait_for_message_stream`].
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
    /// }).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_message_stream`]: Self::wait_for_message_stream
    pub fn wait_for_message<F: Fn(&MessageCreate) -> bool + Send + Sync + 'static>(
        &self,
        channel_id: ChannelId,
        check: impl Into<Box<F>>,
    ) -> WaitForMessageFuture {
        tracing::trace!(%channel_id, "waiting for message in channel");
        let (tx, rx) = oneshot::channel();

        {
            let mut guild = self.0.messages.entry(channel_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Oneshot(tx)),
            });
        }

        WaitForMessageFuture { rx }
    }

    /// Wait for a stream of message in a certain channel.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for only one message matching the given predicate,
    /// use [`wait_for_message`].
    ///
    /// # Examples
    ///
    /// Wait for multiple messages in channel 123 by user 456 with the content
    /// "test":
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::stream::StreamExt;
    /// use twilight_model::{gateway::payload::MessageCreate, id::{ChannelId, UserId}};
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let mut messages = standby.wait_for_message_stream(ChannelId(123), |event: &MessageCreate| {
    ///     event.author.id == UserId(456) && event.content == "test"
    /// });
    ///
    /// while let Some(message) = messages.next().await {
    ///     println!("got message by {}", message.author.id);
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_message`]: Self::wait_for_message
    pub fn wait_for_message_stream<F: Fn(&MessageCreate) -> bool + Send + Sync + 'static>(
        &self,
        channel_id: ChannelId,
        check: impl Into<Box<F>>,
    ) -> WaitForMessageStream {
        tracing::trace!(%channel_id, "waiting for message in channel");
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut guild = self.0.messages.entry(channel_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Mpsc(tx)),
            });
        }

        WaitForMessageStream { rx }
    }

    /// Wait for a reaction on a certain message.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for multiple reactions matching the given predicate,
    /// use [`wait_for_reaction_stream`].
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
    /// }).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_reaction_stream`]: Self::wait_for_reaction_stream
    pub fn wait_for_reaction<F: Fn(&ReactionAdd) -> bool + Send + Sync + 'static>(
        &self,
        message_id: MessageId,
        check: impl Into<Box<F>>,
    ) -> WaitForReactionFuture {
        tracing::trace!(%message_id, "waiting for reaction on message");
        let (tx, rx) = oneshot::channel();

        {
            let mut guild = self.0.reactions.entry(message_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Oneshot(tx)),
            });
        }

        WaitForReactionFuture { rx }
    }

    /// Wait for a stream of reactions on a certain message.
    ///
    /// Returns a `Canceled` error if the `Standby` struct was dropped.
    ///
    /// If you need to wait for only one reaction matching the given predicate,
    /// use [`wait_for_reaction`].
    ///
    /// # Examples
    ///
    /// Wait for multiple reactions on message 123 with unicode reaction "🤠":
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use futures_util::stream::StreamExt;
    /// use twilight_model::{
    ///     channel::ReactionType,
    ///     gateway::payload::ReactionAdd,
    ///     id::{MessageId, UserId},
    /// };
    /// use twilight_standby::Standby;
    ///
    /// let standby = Standby::new();
    ///
    /// let mut reactions = standby.wait_for_reaction_stream(MessageId(123), |event: &ReactionAdd| {
    ///     matches!(&event.emoji, ReactionType::Unicode { name } if name == "🤠")
    /// });
    ///
    /// while let Some(reaction) = reactions.next().await {
    ///     println!("got a reaction by {}", reaction.user_id);
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// [`wait_for_reaction`]: Self::wait_for_reaction
    pub fn wait_for_reaction_stream<F: Fn(&ReactionAdd) -> bool + Send + Sync + 'static>(
        &self,
        message_id: MessageId,
        check: impl Into<Box<F>>,
    ) -> WaitForReactionStream {
        tracing::trace!(%message_id, "waiting for reaction on message");
        let (tx, rx) = mpsc::unbounded_channel();

        {
            let mut guild = self.0.reactions.entry(message_id).or_default();
            guild.push(Bystander {
                func: check.into(),
                sender: Some(Sender::Mpsc(tx)),
            });
        }

        WaitForReactionStream { rx }
    }

    fn next_event_id(&self) -> u64 {
        self.0.event_counter.fetch_add(1, Ordering::SeqCst)
    }

    #[tracing::instrument(level = "trace")]
    fn process_event(&self, event: &Event) {
        tracing::trace!(?event, event_type = ?event.kind(), "processing event");

        self.0.events.retain(|id, bystander| {
            // `bystander_process` returns whether it is fulfilled, so invert it
            // here. If it's fulfilled, then we don't want to retain it.
            let retaining = !self.bystander_process(bystander, event);

            tracing::trace!(bystander_id = id, %retaining, "event bystander processed");

            retaining
        });
    }

    #[tracing::instrument(level = "trace")]
    fn process_guild(&self, guild_id: GuildId, event: &Event) {
        let remove = match self.0.guilds.get_mut(&guild_id) {
            Some(mut bystanders) => {
                self.bystander_iter(&mut bystanders, event);

                bystanders.is_empty()
            }
            None => {
                tracing::trace!(%guild_id, "guild has no event bystanders");

                return;
            }
        };

        if remove {
            tracing::trace!(%guild_id, "removing guild from map");

            self.0.guilds.remove(&guild_id);
        }
    }

    #[tracing::instrument(level = "trace")]
    fn process_message(&self, channel_id: ChannelId, event: &MessageCreate) {
        tracing::trace!(%channel_id, "processing message bystanders in channel");

        let remove = match self.0.messages.get_mut(&channel_id) {
            Some(mut bystanders) => {
                self.bystander_iter(&mut bystanders, event);

                bystanders.is_empty()
            }
            None => {
                tracing::trace!(%channel_id, "channel has no message bystanders");

                return;
            }
        };

        tracing::trace!(%channel_id, %remove, "bystanders processed");

        if remove {
            tracing::trace!(%channel_id, "removing channel");

            self.0.messages.remove(&channel_id);
        }
    }

    fn process_reaction(&self, message_id: MessageId, event: &ReactionAdd) {
        let remove = match self.0.reactions.get_mut(&message_id) {
            Some(mut bystanders) => {
                self.bystander_iter(&mut bystanders, event);

                bystanders.is_empty()
            }
            None => {
                tracing::trace!("message {} has no reaction bystanders", message_id);

                return;
            }
        };

        if remove {
            tracing::trace!("removing message {}", message_id);
            self.0.reactions.remove(&message_id);
        }
    }

    /// Iterate over bystanders and remove the ones that match the predicate.
    #[tracing::instrument(level = "trace")]
    fn bystander_iter<E: Clone + Debug>(&self, bystanders: &mut Vec<Bystander<E>>, event: &E) {
        tracing::trace!(?bystanders, "iterating over bystanders");

        let mut idx = 0;

        while idx < bystanders.len() {
            tracing::trace!(%idx, "checking bystander");
            let bystander = &mut bystanders[idx];

            if self.bystander_process(bystander, event) {
                tracing::trace!(%idx, "removing bystander in list");

                bystanders.remove(idx);
            } else {
                tracing::trace!("retaining bystander");

                idx += 1;
            }
        }
    }

    /// Process a bystander, sending the event if the sender is active and the
    /// predicate matches. Returns whether the bystander has fulfilled.
    ///
    /// Returns `true` if the bystander is fulfilled, meaning that the channel
    /// is now closed or the predicate matched and the event closed.
    #[tracing::instrument(level = "trace")]
    fn bystander_process<E: Clone + Debug>(&self, bystander: &mut Bystander<E>, event: &E) -> bool {
        let sender = match bystander.sender.take() {
            Some(sender) => sender,
            None => {
                tracing::trace!("bystander has no sender, indicating for removal");

                return true;
            }
        };

        if sender.is_closed() {
            tracing::trace!("bystander's rx dropped, indicating for removal");

            return true;
        }

        if !(bystander.func)(event) {
            tracing::trace!("bystander check doesn't match, not removing");
            bystander.sender.replace(sender);

            return false;
        }

        match sender {
            Sender::Oneshot(tx) => {
                let _ = tx.send(event.clone());
                tracing::trace!("bystander matched event, indicating for removal");

                true
            }
            Sender::Mpsc(tx) => {
                if tx.send(event.clone()).is_ok() {
                    tracing::trace!("bystander is a stream, retaining in map");

                    bystander.sender.replace(Sender::Mpsc(tx));

                    false
                } else {
                    true
                }
            }
        }
    }
}

const fn event_guild_id(event: &Event) -> Option<GuildId> {
    match event {
        Event::BanAdd(e) => Some(e.guild_id),
        Event::BanRemove(e) => Some(e.guild_id),
        Event::ChannelCreate(e) => channel_guild_id(&e.0),
        Event::ChannelDelete(e) => channel_guild_id(&e.0),
        Event::ChannelPinsUpdate(_) => None,
        Event::ChannelUpdate(e) => channel_guild_id(&e.0),
        Event::GatewayHeartbeatAck => None,
        Event::GatewayHeartbeat(_) => None,
        Event::GatewayHello(_) => None,
        Event::GatewayInvalidateSession(_) => None,
        Event::GatewayReconnect => None,
        Event::GiftCodeUpdate => None,
        Event::GuildCreate(e) => Some(e.0.id),
        Event::GuildDelete(e) => Some(e.id),
        Event::GuildEmojisUpdate(e) => Some(e.guild_id),
        Event::GuildIntegrationsUpdate(e) => Some(e.guild_id),
        Event::GuildUpdate(e) => Some(e.0.id),
        Event::IntegrationCreate(e) => e.0.guild_id,
        Event::IntegrationDelete(e) => Some(e.guild_id),
        Event::IntegrationUpdate(e) => e.0.guild_id,
        Event::InteractionCreate(e) => e.0.guild_id(),
        Event::InviteCreate(e) => Some(e.guild_id),
        Event::InviteDelete(e) => Some(e.guild_id),
        Event::MemberAdd(e) => Some(e.0.guild_id),
        Event::MemberChunk(e) => Some(e.guild_id),
        Event::MemberRemove(e) => Some(e.guild_id),
        Event::MemberUpdate(e) => Some(e.guild_id),
        Event::MessageCreate(e) => e.0.guild_id,
        Event::MessageDelete(_) => None,
        Event::MessageDeleteBulk(_) => None,
        Event::MessageUpdate(_) => None,
        Event::PresenceUpdate(e) => Some(e.guild_id),
        Event::PresencesReplace => None,
        Event::ReactionAdd(e) => e.0.guild_id,
        Event::ReactionRemove(e) => e.0.guild_id,
        Event::ReactionRemoveAll(e) => e.guild_id,
        Event::ReactionRemoveEmoji(e) => Some(e.guild_id),
        Event::Ready(_) => None,
        Event::Resumed => None,
        Event::RoleCreate(e) => Some(e.guild_id),
        Event::RoleDelete(e) => Some(e.guild_id),
        Event::RoleUpdate(e) => Some(e.guild_id),
        Event::ShardConnected(_) => None,
        Event::ShardConnecting(_) => None,
        Event::ShardDisconnected(_) => None,
        Event::ShardIdentifying(_) => None,
        Event::ShardPayload(_) => None,
        Event::ShardReconnecting(_) => None,
        Event::ShardResuming(_) => None,
        Event::StageInstanceCreate(e) => Some(e.0.guild_id),
        Event::StageInstanceDelete(e) => Some(e.0.guild_id),
        Event::StageInstanceUpdate(e) => Some(e.0.guild_id),
        Event::TypingStart(e) => e.guild_id,
        Event::UnavailableGuild(e) => Some(e.id),
        Event::UserUpdate(_) => None,
        Event::VoiceServerUpdate(e) => e.guild_id,
        Event::VoiceStateUpdate(e) => e.0.guild_id,
        Event::WebhooksUpdate(e) => Some(e.guild_id),
    }
}

const fn channel_guild_id(channel: &Channel) -> Option<GuildId> {
    match channel {
        Channel::Guild(c) => c.guild_id(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Standby;
    use futures_util::StreamExt;
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;
    use twilight_model::{
        channel::{
            message::{Message, MessageType},
            Reaction, ReactionType,
        },
        gateway::{
            event::{Event, EventType},
            payload::{MessageCreate, ReactionAdd, Ready, RoleDelete},
        },
        id::{ApplicationId, ChannelId, GuildId, MessageId, RoleId, UserId},
        oauth::PartialApplication,
        user::{CurrentUser, User, UserFlags},
    };

    assert_impl_all!(Standby: Clone, Debug, Default, Send, Sync);

    fn message() -> Message {
        Message {
            id: MessageId(3),
            activity: None,
            application: None,
            application_id: None,
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
            components: Vec::new(),
            content: "test".to_owned(),
            edited_timestamp: None,
            embeds: Vec::new(),
            flags: None,
            guild_id: Some(GuildId(4)),
            interaction: None,
            kind: MessageType::Regular,
            member: None,
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
        }
    }

    fn reaction() -> Reaction {
        Reaction {
            channel_id: ChannelId(2),
            emoji: ReactionType::Unicode {
                name: "🍎".to_owned(),
            },
            guild_id: Some(GuildId(1)),
            member: None,
            message_id: MessageId(4),
            user_id: UserId(3),
        }
    }

    #[tokio::test]
    async fn test_wait_for() {
        let standby = Standby::new();
        let wait = standby.wait_for(GuildId(1), |event: &Event| match event {
            Event::RoleDelete(e) => e.guild_id == GuildId(1),
            _ => false,
        });
        standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(2),
        }));

        assert!(matches!(
            wait.await,
            Ok(Event::RoleDelete(RoleDelete {
                guild_id: GuildId(1),
                role_id: RoleId(2),
            }))
        ));
        assert!(standby.0.guilds.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_stream() {
        let standby = Standby::new();
        let mut stream = standby.wait_for_stream(
            GuildId(1),
            |event: &Event| matches!(event, Event::RoleDelete(e) if e.guild_id.0 == 1),
        );
        standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(2),
        }));
        standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(3),
        }));

        assert!(matches!(
            stream.next().await,
            Some(Event::RoleDelete(RoleDelete {
                guild_id: GuildId(1),
                role_id: RoleId(2),
            }))
        ));
        assert!(matches!(
            stream.next().await,
            Some(Event::RoleDelete(RoleDelete {
                guild_id: GuildId(1),
                role_id: RoleId(3),
            }))
        ));
        assert!(!standby.0.guilds.is_empty());
        drop(stream);
        standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(4),
        }));
        assert!(standby.0.guilds.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_event() {
        let ready = Ready {
            application: PartialApplication {
                flags: UserFlags::empty(),
                id: ApplicationId(0),
            },
            guilds: Vec::new(),
            session_id: String::new(),
            shard: Some([5, 7]),
            user: CurrentUser {
                avatar: None,
                bot: false,
                discriminator: "0001".to_owned(),
                email: None,
                id: UserId(1),
                mfa_enabled: true,
                name: "twilight".to_owned(),
                verified: Some(false),
                premium_type: None,
                public_flags: None,
                flags: None,
                locale: None,
            },
            version: 6,
        };
        let event = Event::Ready(Box::new(ready));

        let standby = Standby::new();
        let wait = standby.wait_for_event(|event: &Event| match event {
            Event::Ready(ready) => ready.shard.map(|[id, _]| id == 5).unwrap_or(false),
            _ => false,
        });
        assert!(!standby.0.events.is_empty());
        standby.process(&event);

        assert_eq!(event, wait.await.unwrap());
        assert!(standby.0.events.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_event_stream() {
        let standby = Standby::new();
        let mut stream =
            standby.wait_for_event_stream(|event: &Event| event.kind() == EventType::Resumed);
        standby.process(&Event::Resumed);
        assert_eq!(stream.next().await, Some(Event::Resumed));
        assert!(!standby.0.events.is_empty());
        drop(stream);
        standby.process(&Event::Resumed);
        assert!(standby.0.events.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_message() {
        let message = message();
        let event = Event::MessageCreate(Box::new(MessageCreate(message)));

        let standby = Standby::new();
        let wait = standby.wait_for_message(ChannelId(1), |message: &MessageCreate| {
            message.author.id == UserId(2)
        });
        standby.process(&event);

        assert_eq!(MessageId(3), wait.await.map(|msg| msg.id).unwrap());
        assert!(standby.0.messages.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_message_stream() {
        let standby = Standby::new();
        let mut stream = standby.wait_for_message_stream(ChannelId(1), |_: &MessageCreate| true);
        standby.process(&Event::MessageCreate(Box::new(MessageCreate(message()))));
        standby.process(&Event::MessageCreate(Box::new(MessageCreate(message()))));

        assert!(stream.next().await.is_some());
        assert!(stream.next().await.is_some());
        drop(stream);
        assert_eq!(1, standby.0.messages.len());
        standby.process(&Event::MessageCreate(Box::new(MessageCreate(message()))));
        assert!(standby.0.messages.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_reaction() {
        let event = Event::ReactionAdd(Box::new(ReactionAdd(reaction())));

        let standby = Standby::new();
        let wait = standby.wait_for_reaction(MessageId(4), |reaction: &ReactionAdd| {
            reaction.user_id == UserId(3)
        });

        standby.process(&event);

        assert_eq!(
            UserId(3),
            wait.await.map(|reaction| reaction.user_id).unwrap()
        );
        assert!(standby.0.reactions.is_empty());
    }

    #[tokio::test]
    async fn test_wait_for_reaction_stream() {
        let standby = Standby::new();
        let mut stream = standby.wait_for_reaction_stream(MessageId(4), |_: &ReactionAdd| true);
        standby.process(&Event::ReactionAdd(Box::new(ReactionAdd(reaction()))));
        standby.process(&Event::ReactionAdd(Box::new(ReactionAdd(reaction()))));

        assert!(stream.next().await.is_some());
        assert!(stream.next().await.is_some());
        drop(stream);
        assert_eq!(1, standby.0.reactions.len());
        standby.process(&Event::ReactionAdd(Box::new(ReactionAdd(reaction()))));
        assert!(standby.0.reactions.is_empty());
    }

    #[tokio::test]
    async fn test_handles_wrong_events() {
        let standby = Standby::new();
        let wait = standby.wait_for_event(|event: &Event| event.kind() == EventType::Resumed);

        standby.process(&Event::PresencesReplace);
        standby.process(&Event::PresencesReplace);
        standby.process(&Event::Resumed);

        assert_eq!(Event::Resumed, wait.await.unwrap());
    }

    /// Test that generic event handlers will be given the opportunity to
    /// process events with specific handlers (message creates, reaction adds)
    /// and guild events. Similarly, guild handlers should be able to process
    /// specific handler events as well.
    #[tokio::test]
    async fn test_process_nonspecific_handling() {
        let standby = Standby::new();

        // generic event handler gets message creates
        let wait = standby.wait_for_event(|event: &Event| event.kind() == EventType::MessageCreate);
        standby.process(&Event::MessageCreate(Box::new(MessageCreate(message()))));
        assert!(matches!(wait.await, Ok(Event::MessageCreate(_))));

        // generic event handler gets reaction adds
        let wait = standby.wait_for_event(|event: &Event| event.kind() == EventType::ReactionAdd);
        standby.process(&Event::ReactionAdd(Box::new(ReactionAdd(reaction()))));
        assert!(matches!(wait.await, Ok(Event::ReactionAdd(_))));

        // generic event handler gets other guild events
        let wait = standby.wait_for_event(|event: &Event| event.kind() == EventType::RoleDelete);
        standby.process(&Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(2),
        }));
        assert!(matches!(wait.await, Ok(Event::RoleDelete(_))));

        // guild event handler gets message creates or reaction events
        let wait = standby.wait_for(GuildId(1), |event: &Event| {
            event.kind() == EventType::ReactionAdd
        });
        standby.process(&Event::ReactionAdd(Box::new(ReactionAdd(reaction()))));
        assert!(matches!(wait.await, Ok(Event::ReactionAdd(_))));
    }
}
