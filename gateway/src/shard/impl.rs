use super::{
    config::ShardConfig,
    error::{Error, Result},
    event::Events,
    processor::{Latency, Session, ShardProcessor},
    sink::ShardSink,
    stage::Stage,
};
use crate::{listener::Listeners, EventTypeFlags};
use futures::{
    future::{self, AbortHandle},
    Stream,
};

use log::debug;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::watch::Receiver as WatchReceiver;

use std::borrow::Cow;
use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};
use tokio_tungstenite::tungstenite::Message;
use twilight_model::gateway::event::Event;

#[derive(Debug)]
pub struct ShardRef {
    config: Arc<ShardConfig>,
    listeners: Listeners<Event>,
    processor_handle: AbortHandle,
    session: WatchReceiver<Arc<Session>>,
}

/// Information about a shard, including its latency, current session sequence,
/// and connection stage.
#[derive(Clone, Debug)]
pub struct Information {
    id: u64,
    latency: Latency,
    seq: u64,
    stage: Stage,
}

impl Information {
    /// Returns the ID of the shard.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the latency information for the shard.
    ///
    /// This includes the average latency over all time, and the latency
    /// information for the 5 most recent heartbeats.
    pub fn latency(&self) -> &Latency {
        &self.latency
    }

    /// The current sequence of the connection.
    ///
    /// This is the number of the event that was received this session (without
    /// reconnecting). A larger number typically correlates that the shard has
    /// been connected for a longer time, while a smaller number typically
    /// correlates to meaning that it's been connected for a less amount of
    /// time.
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// The current stage of the shard.
    ///
    /// For example, once a shard is fully booted then it will be
    /// [`Connected`].
    ///
    /// [`Connected`]: enum.Stage.html
    pub fn stage(&self) -> Stage {
        self.stage
    }
}
/// Holds the sessions id and sequence number to resume this shard's session with with
#[derive(Clone, Debug)]
pub struct ResumeSession {
    pub session_id: String,
    pub sequence: u64,
}

#[derive(Clone, Debug)]
pub struct Shard(Arc<ShardRef>);

impl Shard {
    /// Creates a new shard, which will automatically connect to the gateway.
    ///
    /// # Examples
    ///
    /// Create a new shard, wait a second, and then print its current connection
    /// stage:
    ///
    /// ```no_run
    /// use twilight_gateway::Shard;
    /// use std::{env, time::Duration};
    /// use tokio::time as tokio_time;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let shard = Shard::new(env::var("DISCORD_TOKEN")?).await?;
    ///
    /// tokio_time::delay_for(Duration::from_secs(1)).await;
    ///
    /// let info = shard.info().await;
    /// println!("Shard stage: {}", info.stage());
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Errors if the `ShardProcessor` could not be started.
    pub async fn new(config: impl Into<ShardConfig>) -> Result<Self> {
        Self::_new(config.into()).await
    }

    async fn _new(config: ShardConfig) -> Result<Self> {
        let config = Arc::new(config);

        let url = config
            .http_client()
            .gateway()
            .authed()
            .await
            .map_err(|source| Error::GettingGatewayUrl { source })?
            .url;

        let (processor, wrx) = ShardProcessor::new(Arc::clone(&config), url).await?;
        let listeners = processor.listeners.clone();
        let (fut, handle) = future::abortable(processor.run());

        tokio::spawn(async move {
            let _ = fut.await;

            debug!("[Shard] Shard processor future ended");
        });

        Ok(Self(Arc::new(ShardRef {
            config,
            listeners,
            processor_handle: handle,
            session: wrx,
        })))
    }

    /// Returns an immutable reference to the configuration used for this client.
    pub fn config(&self) -> &ShardConfig {
        &self.0.config
    }

    /// Returns information about the running of the shard, such as the current
    /// connection stage.
    pub async fn info(&self) -> Information {
        let session = self.session();

        Information {
            id: self.config().shard()[0],
            latency: session.heartbeats.latency().await,
            seq: session.seq(),
            stage: session.stage(),
        }
    }

    /// Returns a handle to the current session
    ///
    /// # Note
    ///
    /// This session can be invalidated if it is kept around
    /// under a reconnect or resume. In consequence this call
    /// should not be cached.
    pub fn session(&self) -> Arc<Session> {
        Arc::clone(&self.0.session.borrow())
    }

    /// Creates a new stream of events from the shard.
    ///
    /// There can be multiple streams of events. All events will be broadcast to
    /// all streams of events.
    ///
    /// All event types except for [`EventType::SHARD_PAYLOAD`] are enabled.
    /// If you need to enable it, consider calling [`some_events`] instead.
    ///
    /// [`EventType::SHARD_PAYLOAD`]: events/struct.EventType.html#const.SHARD_PAYLOAD
    /// [`some_events`]: #method.some_events
    pub async fn events(&self) -> impl Stream<Item = Event> {
        let rx = self.0.listeners.add(EventTypeFlags::default()).await;

        Events::new(EventTypeFlags::default(), rx)
    }

    /// Creates a new filtered stream of events from the shard.
    ///
    /// Only the events specified in the bitflags will be sent over the stream.
    ///
    /// # Examples
    ///
    /// Filter the events so that you only receive the [`Event::ShardConnected`]
    /// and [`Event::ShardDisconnected`] events:
    ///
    /// ```no_run
    /// use twilight_gateway::{EventTypeFlags, Event, Shard};
    /// use futures::StreamExt;
    /// use std::env;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let shard = Shard::new(env::var("DISCORD_TOKEN")?).await?;
    ///
    /// let event_types = EventTypeFlags::SHARD_CONNECTED | EventTypeFlags::SHARD_DISCONNECTED;
    /// let mut events = shard.some_events(event_types).await;
    ///
    /// while let Some(event) = events.next().await {
    ///     match event {
    ///         Event::ShardConnected(_) => println!("Shard is now connected"),
    ///         Event::ShardDisconnected(_) => println!("Shard is now disconnected"),
    ///         // No other event will come in through the stream.
    ///         _ => {},
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    pub async fn some_events(&self, event_types: EventTypeFlags) -> impl Stream<Item = Event> {
        let rx = self.0.listeners.add(event_types).await;

        Events::new(event_types, rx)
    }

    /// Returns an interface implementing the `Sink` trait which can be used to
    /// send messages.
    ///
    /// # Note
    ///
    /// This call should not be cached for too long
    /// as it will be invalidated by reconnects and
    /// resumes.
    pub fn sink(&self) -> ShardSink {
        let session = self.session();

        ShardSink(session.tx.clone())
    }

    /// Send a command over the gateway.
    ///
    /// # Errors
    /// Fails if command could not be serialized, or if the command could
    /// not be sent.
    pub async fn command(&self, com: &impl serde::Serialize) -> Result<()> {
        let payload = Message::Text(
            crate::json_to_string(&com)
                .map_err(|err| Error::PayloadSerialization { source: err })?,
        );
        let session = self.session();

        // Tick ratelimiter.
        session.ratelimit.lock().await.tick().await;

        session
            .tx
            .unbounded_send(payload)
            .map_err(|err| Error::SendingMessage { source: err })?;
        Ok(())
    }

    /// Shuts down the shard.
    ///
    /// This will cleanly close the connection, causing discord to end the session and show the bot offline
    pub async fn shutdown(&self) {
        let session = self.session();
        // Since we're shutting down now, we don't care if it sends or not.
        let _ = session.tx.unbounded_send(Message::Close(None));

        self.0.processor_handle.abort();
        self.0.listeners.remove_all().await;
        session.stop_heartbeater().await;
    }

    /// This will shut down the shard in a resumable way and return shard id and optional session info to resume with later if this shard is resumable
    pub async fn shutdown_resumable(&self) -> (u64, Option<ResumeSession>) {
        let session = self.session();
        let _ = session.tx.unbounded_send(Message::Close(Some(CloseFrame {
            code: CloseCode::Restart,
            reason: Cow::from("Closing in a resumable way"),
        })));
        let shard_id = self.config().shard[0];
        let session_id = session.id.lock().await.clone();
        let sequence = session.seq.load(Ordering::Relaxed);

        self.0.processor_handle.abort();
        self.0.listeners.remove_all().await;
        session.stop_heartbeater().await;

        let data = match session_id {
            Some(id) => Some(ResumeSession {
                session_id: id,
                sequence,
            }),
            None => None,
        };

        (shard_id, data)
    }
}
