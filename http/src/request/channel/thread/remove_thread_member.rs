use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::id::{ChannelId, UserId};

/// Remove another member from a thread.
///
/// Requires that the thread is not archived.
///
/// Requires the [`MANAGE_THREADS`] permission, unless both the thread is a
/// [`GuildPrivateThread`], and the current user is the creator of the thread.
///
/// [`GuildPrivateThread`]: twilight_model::channel::ChannelType::GuildPrivateThread
/// [`MANAGE_THREADS`]: twilight_model::guild::Permissions::MANAGE_THREADS
pub struct RemoveThreadMember<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> RemoveThreadMember<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId, user_id: UserId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
            user_id,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::from_route(Route::RemoveThreadMember {
            channel_id: self.channel_id.0,
            user_id: self.user_id.0,
        });

        self.fut.replace(Box::pin(self.http.verify(request)));

        Ok(())
    }
}

poll_req!(RemoveThreadMember<'_>, ());
