use super::RequestReactionType;
use crate::{
    client::Client,
    request::{validate, Request},
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    id::{ChannelId, MessageId, UserId},
    user::User,
};

/// The error created if the reactions can not be retrieved as configured.
#[derive(Debug)]
pub struct GetReactionsError {
    kind: GetReactionsErrorType,
}

impl GetReactionsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetReactionsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (GetReactionsErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for GetReactionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetReactionsErrorType::LimitInvalid { .. } => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetReactionsError {}

/// Type of [`GetReactionsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetReactionsErrorType {
    /// The number of reactions to retrieve must be between 1 and 100, inclusive.
    LimitInvalid {
        /// The provided maximum number of reactions to get.
        limit: u64,
    },
}

struct GetReactionsFields {
    after: Option<UserId>,
    limit: Option<u64>,
}

/// Get a list of users that reacted to a message with an `emoji`.
///
/// This endpoint is limited to 100 users maximum, so if a message has more than 100 reactions,
/// requests must be chained until all reactions are retireved.
pub struct GetReactions<'a> {
    channel_id: ChannelId,
    emoji: &'a RequestReactionType<'a>,
    fields: GetReactionsFields,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> GetReactions<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &'a RequestReactionType<'a>,
    ) -> Self {
        Self {
            channel_id,
            emoji,
            fields: GetReactionsFields {
                after: None,
                limit: None,
            },
            http,
            message_id,
        }
    }

    /// Get users after this id.
    pub const fn after(mut self, after: UserId) -> Self {
        self.fields.after = Some(after);

        self
    }

    /// Set the maximum number of users to retrieve.
    ///
    /// The minimum is 1 and the maximum is 100. If no limit is specified, Discord sets the default
    /// to 25.
    ///
    /// # Errors
    ///
    /// Returns a [`GetReactionsErrorType::LimitInvalid`] error type if the
    /// amount is greater than 100.
    pub const fn limit(mut self, limit: u64) -> Result<Self, GetReactionsError> {
        if !validate::get_reactions_limit(limit) {
            return Err(GetReactionsError {
                kind: GetReactionsErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit = Some(limit);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<User>> {
        let request = Request::from_route(Route::GetReactionUsers {
            after: self.fields.after.map(|x| x.0),
            channel_id: self.channel_id.0,
            emoji: self.emoji,
            limit: self.fields.limit,
            message_id: self.message_id.0,
        });

        self.http.request(request)
    }
}
