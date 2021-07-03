use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::ChannelId;

/// Delete the stage instance of a stage channel.
///
/// Requires the user to be a moderator of the stage channel.
pub struct DeleteStageInstance<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> DeleteStageInstance<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::DeleteStageInstance {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
