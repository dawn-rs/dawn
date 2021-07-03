use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{channel::Webhook, id::ChannelId};

/// Get all the webhooks of a channel.
pub struct GetChannelWebhooks<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> GetChannelWebhooks<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Webhook>> {
        let request = Request::from_route(Route::GetChannelWebhooks {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
