use super::RequestReactionType;
use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{ChannelId, MessageId};

/// Delete one reaction by a user on a message.
pub struct DeleteReaction<'a> {
    channel_id: ChannelId,
    emoji: RequestReactionType,
    http: &'a Client,
    message_id: MessageId,
    target_user: String,
}

impl<'a> DeleteReaction<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
        target_user: impl Into<String>,
    ) -> Self {
        Self {
            channel_id,
            emoji,
            http,
            message_id,
            target_user: target_user.into(),
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::DeleteReaction {
            channel_id: self.channel_id.0,
            emoji: self.emoji.display().to_string(),
            message_id: self.message_id.0,
            user: self.target_user,
        });

        self.http.request(request)
    }
}
