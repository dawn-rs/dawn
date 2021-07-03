use crate::{
    client::Client,
    request::{validate, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::{GuildId, UserId};

/// The error created when the ban can not be created as configured.
#[derive(Debug)]
pub struct CreateBanError {
    kind: CreateBanErrorType,
}

impl CreateBanError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateBanErrorType {
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
    pub fn into_parts(self) -> (CreateBanErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for CreateBanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateBanErrorType::DeleteMessageDaysInvalid => {
                f.write_str("the number of days' worth of messages to delete is invalid")
            }
        }
    }
}

impl Error for CreateBanError {}

/// Type of [`CreateBanError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateBanErrorType {
    /// The number of days' worth of messages to delete is greater than 7.
    DeleteMessageDaysInvalid,
}

struct CreateBanFields<'a> {
    delete_message_days: Option<u64>,
    reason: Option<&'a str>,
}

/// Bans a user from a guild, optionally with the number of days' worth of
/// messages to delete and the reason.
///
/// # Examples
///
/// Ban user `200` from guild `100`, deleting
/// 1 day's worth of messages, for the reason `"memes"`:
///
/// ```rust,no_run
/// use twilight_http::{request::AuditLogReason, Client};
/// use twilight_model::id::{GuildId, UserId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = GuildId(100);
/// let user_id = UserId(200);
/// client.create_ban(guild_id, user_id)
///     .delete_message_days(1)?
///     .reason("memes")?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct CreateBan<'a> {
    fields: CreateBanFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> CreateBan<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            fields: CreateBanFields {
                delete_message_days: None,
                reason: None,
            },
            guild_id,
            http,
            user_id,
        }
    }

    /// Set the number of days' worth of messages to delete.
    ///
    /// The number of days must be less than or equal to 7.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateBanErrorType::DeleteMessageDaysInvalid`] error type if
    /// the number of days is greater than 7.
    pub const fn delete_message_days(mut self, days: u64) -> Result<Self, CreateBanError> {
        if !validate::ban_delete_message_days(days) {
            return Err(CreateBanError {
                kind: CreateBanErrorType::DeleteMessageDaysInvalid,
            });
        }

        self.fields.delete_message_days = Some(days);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::CreateBan {
            delete_message_days: self.fields.delete_message_days,
            guild_id: self.guild_id.0,
            reason: self.fields.reason,
            user_id: self.user_id.0,
        });

        self.http.request(request)
    }
}

impl<'a> AuditLogReason<'a> for CreateBan<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.fields
            .reason
            .replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
