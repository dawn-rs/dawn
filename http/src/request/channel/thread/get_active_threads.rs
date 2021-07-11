use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::{channel::thread::ThreadsListing, id::ChannelId};

/// Returns all active threads in the channel.
///
/// Includes public and private threads. Threads are ordered by their ID in
/// descending order.
pub struct GetActiveThreads<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, ThreadsListing>>,
    http: &'a Client,
}

impl<'a> GetActiveThreads<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::GetActiveThreads {
            channel_id: self.channel_id.0,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetActiveThreads<'_>, ThreadsListing);
