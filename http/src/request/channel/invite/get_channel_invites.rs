use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{id::ChannelId, invite::Invite};

/// Get the invites for a guild channel.
///
/// Requires the [`MANAGE_CHANNELS`] permission. This method only works if the
/// channel is of type [`GuildChannel`].
///
/// [`MANAGE_CHANNELS`]: twilight_model::guild::Permissions::MANAGE_CHANNELS
/// [`GuildChannel`]: twilight_model::channel::GuildChannel
pub struct GetChannelInvites<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> GetChannelInvites<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Invite>> {
        let request = Request::from_route(Route::GetChannelInvites {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
