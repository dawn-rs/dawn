use super::RequestReactionType;
use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{ChannelId, MessageId};

/// Remove all reactions of a specified emoji from a message.
pub struct DeleteAllReaction<'a> {
    channel_id: ChannelId,
    emoji: &'a RequestReactionType<'a>,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> DeleteAllReaction<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &'a RequestReactionType<'a>,
    ) -> Self {
        Self {
            channel_id,
            emoji,
            http,
            message_id,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::DeleteMessageSpecificReaction {
            channel_id: self.channel_id.0,
            message_id: self.message_id.0,
            emoji: self.emoji,
        });

        self.http.request(request)
    }
}
