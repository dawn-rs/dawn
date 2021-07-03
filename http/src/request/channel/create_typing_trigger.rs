use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::ChannelId;

/// Fire a Typing Start event in the channel.
pub struct CreateTypingTrigger<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> CreateTypingTrigger<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::CreateTypingTrigger {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
