use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::{channel::thread::ThreadMember, id::ChannelId};

/// Returns the [`ThreadMember`]s of the thread.
///
/// [`ThreadMember`]: twilight_model::channel::thread::ThreadMember
pub struct GetThreadMembers<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, Vec<ThreadMember>>>,
    http: &'a Client,
}

impl<'a> GetThreadMembers<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::GetThreadMembers {
            channel_id: self.channel_id.0,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetThreadMembers<'_>, Vec<ThreadMember>);
