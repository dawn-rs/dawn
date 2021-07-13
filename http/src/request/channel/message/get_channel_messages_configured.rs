use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, Pending, Request},
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
pub struct GetChannelMessagesConfiguredError {
    kind: GetChannelMessagesConfiguredErrorType,
}

impl GetChannelMessagesConfiguredError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetChannelMessagesConfiguredErrorType {
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
        GetChannelMessagesConfiguredErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetChannelMessagesConfiguredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetChannelMessagesConfiguredErrorType::LimitInvalid { .. } => {
                f.write_str("the limit is invalid")
            }
        }
    }
}

impl Error for GetChannelMessagesConfiguredError {}

/// Type of [`GetChannelMessagesConfiguredError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetChannelMessagesConfiguredErrorType {
    /// The maximum number of messages to retrieve is either 0 or more than 100.
    LimitInvalid {
        /// Provided maximum number of messages to retrieve.
        limit: u64,
    },
}

struct GetChannelMessagesConfiguredFields {
    limit: Option<u64>,
}

/// This struct is returned when one of `after`, `around`, or `before` is specified in
/// [`GetChannelMessages`].
///
/// [`GetChannelMessages`]: super::GetChannelMessages
// nb: after, around, and before are mutually exclusive, so we use this
// "configured" request to utilize the type system to prevent these from being
// set in combination.
pub struct GetChannelMessagesConfigured<'a> {
    after: Option<MessageId>,
    around: Option<MessageId>,
    before: Option<MessageId>,
    channel_id: ChannelId,
    fields: GetChannelMessagesConfiguredFields,
    fut: Option<Pending<'a, Vec<Message>>>,
    http: &'a Client,
}

impl<'a> GetChannelMessagesConfigured<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        after: Option<MessageId>,
        around: Option<MessageId>,
        before: Option<MessageId>,
        limit: Option<u64>,
    ) -> Self {
        Self {
            after,
            around,
            before,
            channel_id,
            fields: GetChannelMessagesConfiguredFields { limit },
            fut: None,
            http,
        }
    }

    /// Set the maximum number of messages to retrieve.
    ///
    /// The minimum is 1 and the maximum is 100.
    ///
    /// # Errors
    ///
    /// Returns a [`GetChannelMessagesConfiguredErrorType::LimitInvalid`] error
    /// type if the amount is greater than 21600.
    pub fn limit(mut self, limit: u64) -> Result<Self, GetChannelMessagesConfiguredError> {
        if !validate_inner::get_channel_messages_limit(limit) {
            return Err(GetChannelMessagesConfiguredError {
                kind: GetChannelMessagesConfiguredErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::from_route(Route::GetMessages {
            after: self.after.map(|x| x.0),
            around: self.around.map(|x| x.0),
            before: self.before.map(|x| x.0),
            channel_id: self.channel_id.0,
            limit: self.fields.limit,
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetChannelMessagesConfigured<'_>, Vec<Message>);
