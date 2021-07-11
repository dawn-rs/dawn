use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::{channel::thread::ThreadsListing, id::ChannelId};

/// Returns archived public threads in the channel.
///
/// Requires the [`READ_MESSAGE_HISTORY`] permission.
///
/// Threads are ordered by [`archive_timestamp`] in descending order.
///
/// When called in a [`GuildText`] channel, returns [`GuildPublicThread`]s.
///
/// When called in a [`GuildNews`] channel, returns [`GuildNewsThread`]s.
///
/// [`archive_timestamp`]: twilight_model::channel::thread::ThreadMetadata::archive_timestamp
/// [`GuildNews`]: twilight_model::channel::ChannelType::GuildNews
/// [`GuildNewsThread`]: twilight_model::channel::ChannelType::GuildNewsThread
/// [`GuildPublicThread`]: twilight_model::channel::ChannelType::GuildPublicThread
/// [`GuildText`]: twilight_model::channel::ChannelType::GuildText
/// [`READ_MESSAGE_HISTORY`]: twilight_model::guild::Permissions::READ_MESSAGE_HISTORY
pub struct GetPublicArchivedThreads<'a> {
    before: Option<String>,
    channel_id: ChannelId,
    fut: Option<Pending<'a, ThreadsListing>>,
    http: &'a Client,
    limit: Option<u64>,
}

impl<'a> GetPublicArchivedThreads<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            before: None,
            channel_id,
            fut: None,
            http,
            limit: None,
        }
    }

    /// Return threads before this ISO 8601 timestamp.
    pub fn before(self, before: impl Into<String>) -> Self {
        self._before(before.into())
    }

    fn _before(mut self, before: String) -> Self {
        self.before.replace(before);

        self
    }

    /// Maximum number of threads to return.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit.replace(limit);

        self
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::GetPublicArchivedThreads {
            before: self.before.clone(),
            channel_id: self.channel_id.0,
            limit: self.limit,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetPublicArchivedThreads<'_>, ThreadsListing);
