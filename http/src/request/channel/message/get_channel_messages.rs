use super::GetChannelMessagesConfigured;
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
    channel::Message,
    id::{ChannelId, MessageId},
};

/// The error returned if the request can not be created as configured.
#[derive(Debug)]
pub struct GetChannelMessagesError {
    kind: GetChannelMessagesErrorType,
}

impl GetChannelMessagesError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetChannelMessagesErrorType {
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
    pub fn into_parts(
        self,
    ) -> (
        GetChannelMessagesErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetChannelMessagesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetChannelMessagesErrorType::LimitInvalid => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetChannelMessagesError {}

/// Type of [`GetChannelMessagesError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetChannelMessagesErrorType {
    /// The maximum number of messages to retrieve is either 0 or more than 100.
    LimitInvalid,
}

#[derive(Default)]
struct GetChannelMessagesFields {
    limit: Option<u64>,
}

/// Get channel messages, by [`ChannelId`].
///
/// Only one of [`after`], [`around`], and [`before`] can be specified at a time.
/// Once these are specified, the type returned is [`GetChannelMessagesConfigured`].
///
/// If [`limit`] is unspecified, the default set by Discord is 50.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::{ChannelId, MessageId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
/// let channel_id = ChannelId(123);
/// let message_id = MessageId(234);
///
/// let messages = client
///     .channel_messages(channel_id)
///     .before(message_id)
///     .limit(6u64)?
///     .exec()
///     .await?;
///
/// # Ok(()) }
/// ```
///
/// [`after`]: Self::after
/// [`around`]: Self::around
/// [`before`]: Self::before
/// [`GetChannelMessagesConfigured`]: super::GetChannelMessagesConfigured
/// [`limit`]: Self::limit
pub struct GetChannelMessages<'a> {
    channel_id: ChannelId,
    fields: GetChannelMessagesFields,
    http: &'a Client,
}

impl<'a> GetChannelMessages<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: GetChannelMessagesFields::default(),
            http,
        }
    }

    pub const fn after(self, message_id: MessageId) -> GetChannelMessagesConfigured<'a> {
        GetChannelMessagesConfigured::new(
            self.http,
            self.channel_id,
            Some(message_id),
            None,
            None,
            self.fields.limit,
        )
    }

    pub const fn around(self, message_id: MessageId) -> GetChannelMessagesConfigured<'a> {
        GetChannelMessagesConfigured::new(
            self.http,
            self.channel_id,
            None,
            Some(message_id),
            None,
            self.fields.limit,
        )
    }

    pub const fn before(self, message_id: MessageId) -> GetChannelMessagesConfigured<'a> {
        GetChannelMessagesConfigured::new(
            self.http,
            self.channel_id,
            None,
            None,
            Some(message_id),
            self.fields.limit,
        )
    }

    /// Set the maximum number of messages to retrieve.
    ///
    /// The minimum is 1 and the maximum is 100.
    ///
    /// # Errors
    ///
    /// Returns a [`GetChannelMessagesErrorType::LimitInvalid`] error type if
    /// the amount is less than 1 or greater than 100.
    pub fn limit(mut self, limit: u64) -> Result<Self, GetChannelMessagesError> {
        if !validate::get_channel_messages_limit(limit) {
            return Err(GetChannelMessagesError {
                kind: GetChannelMessagesErrorType::LimitInvalid,
            });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Message>> {
        let request = Request::from_route(Route::GetMessages {
            after: None,
            around: None,
            before: None,
            channel_id: self.channel_id.0,
            limit: self.fields.limit,
        });

        self.http.request(request)
    }
}
