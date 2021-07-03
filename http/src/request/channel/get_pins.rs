use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{channel::Message, id::ChannelId};

/// Get the pins of a channel.
pub struct GetPins<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> GetPins<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Message>> {
        let request = Request::from_route(Route::GetPins {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
