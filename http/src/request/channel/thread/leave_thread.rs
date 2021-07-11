use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::id::ChannelId;

/// Remove the current user from a thread.
///
/// Requires that the thread is not archived.
pub struct LeaveThread<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
}

impl<'a> LeaveThread<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::LeaveThread {
            channel_id: self.channel_id.0,
        });

        self.fut.replace(Box::pin(self.http.verify(request)));

        Ok(())
    }
}

poll_req!(LeaveThread<'_>, ());
