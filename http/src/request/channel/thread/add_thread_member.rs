use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::id::{ChannelId, UserId};

/// Add another member to a thread.
///
/// Requires the ability to send messages in the thread, and that the thread is
/// not archived.
pub struct AddThreadMember<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> AddThreadMember<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId, user_id: UserId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
            user_id,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::AddThreadMember {
            channel_id: self.channel_id.0,
            user_id: self.user_id.0,
        });

        self.fut.replace(Box::pin(self.http.verify(request)));

        Ok(())
    }
}

poll_req!(AddThreadMember<'_>, ());
