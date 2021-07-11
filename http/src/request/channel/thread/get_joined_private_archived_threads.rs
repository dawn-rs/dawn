use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::{channel::thread::ThreadsListing, id::ChannelId};

/// Returns archived private threads in the channel that the current user has
/// joined.
///
/// Threads are ordered by their ID in descending order.
pub struct GetJoinedPrivateArchivedThreads<'a> {
    before: Option<ChannelId>,
    channel_id: ChannelId,
    fut: Option<Pending<'a, ThreadsListing>>,
    http: &'a Client,
    limit: Option<u64>,
}

impl<'a> GetJoinedPrivateArchivedThreads<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            before: None,
            channel_id,
            fut: None,
            http,
            limit: None,
        }
    }

    /// Return threads before this ID.
    pub fn before(mut self, before: ChannelId) -> Self {
        self.before.replace(before);

        self
    }

    /// Maximum number of threads to return.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit.replace(limit);

        self
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::GetJoinedPrivateArchivedThreads {
            before: self.before.map(|id| id.0),
            channel_id: self.channel_id.0,
            limit: self.limit,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetJoinedPrivateArchivedThreads<'_>, ThreadsListing);
