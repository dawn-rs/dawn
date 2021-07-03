use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{
    channel::Message,
    id::{ChannelId, MessageId},
};

/// Get a message by [`ChannelId`] and [`MessageId`].
pub struct GetMessage<'a> {
    channel_id: ChannelId,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> GetMessage<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Self {
        Self {
            channel_id,
            http,
            message_id,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Message> {
        let request = Request::from_route(Route::GetMessage {
            channel_id: self.channel_id.0,
            message_id: self.message_id.0,
        });

        self.http.request(request)
    }
}
