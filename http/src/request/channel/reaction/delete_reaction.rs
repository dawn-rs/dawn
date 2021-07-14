use super::RequestReactionType;
use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{ChannelId, MessageId, UserId};

/// User to delete the reaction of.
pub(crate) enum TargetUser {
    /// Delete a reaction of the current user.
    Current,
    /// Delete a reaction from a user by their ID.
    Id(UserId),
}

/// Delete one reaction by a user on a message.
pub struct DeleteReaction<'a> {
    channel_id: ChannelId,
    emoji: &'a RequestReactionType<'a>,
    http: &'a Client,
    message_id: MessageId,
    target_user: TargetUser,
}

impl<'a> DeleteReaction<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &'a RequestReactionType<'a>,
        target_user: TargetUser,
    ) -> Self {
        Self {
            channel_id,
            emoji,
            http,
            message_id,
            target_user,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let route = match self.target_user {
            TargetUser::Current => Route::DeleteReactionCurrentUser {
                channel_id: self.channel_id.0,
                emoji: self.emoji,
                message_id: self.message_id.0,
            },
            TargetUser::Id(user_id) => Route::DeleteReaction {
                channel_id: self.channel_id.0,
                emoji: self.emoji,
                message_id: self.message_id.0,
                user_id: user_id.0,
            },
        };

        self.http.request(Request::from_route(route))
    }
}
