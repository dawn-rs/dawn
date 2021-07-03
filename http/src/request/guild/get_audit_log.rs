use crate::{
    client::Client,
    request::{validate, Request},
    response::ResponseFuture,
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::audit_log::{AuditLog, AuditLogEvent},
    id::{GuildId, UserId},
};

/// The error returned when the audit log can not be requested as configured.
#[derive(Debug)]
pub struct GetAuditLogError {
    kind: GetAuditLogErrorType,
}

impl GetAuditLogError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetAuditLogErrorType {
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
    pub fn into_parts(self) -> (GetAuditLogErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for GetAuditLogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetAuditLogErrorType::LimitInvalid => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetAuditLogError {}

/// Type of [`GetAuditLogError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetAuditLogErrorType {
    /// The limit is either 0 or more than 100.
    LimitInvalid,
}

struct GetAuditLogFields {
    action_type: Option<AuditLogEvent>,
    before: Option<u64>,
    limit: Option<u64>,
    user_id: Option<UserId>,
}

/// Get the audit log for a guild.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("token".to_owned());
///
/// let guild_id = GuildId(101);
/// let audit_log = client
/// // not done
///     .audit_log(guild_id)
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct GetAuditLog<'a> {
    fields: GetAuditLogFields,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetAuditLog<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: GetAuditLogFields {
                action_type: None,
                before: None,
                limit: None,
                user_id: None,
            },
            guild_id,
            http,
        }
    }

    /// Filter by an action type.
    pub const fn action_type(mut self, action_type: AuditLogEvent) -> Self {
        self.fields.action_type = Some(action_type);

        self
    }

    /// Get audit log entries before the entry specified.
    pub const fn before(mut self, before: u64) -> Self {
        self.fields.before = Some(before);

        self
    }

    /// Set the maximum number of audit logs to retrieve.
    ///
    /// The minimum is 1 and the maximum is 100.
    ///
    /// # Errors
    ///
    /// Returns a [`GetAuditLogErrorType::LimitInvalid`] error type if the
    /// `limit` is 0 or greater than 100.
    pub const fn limit(mut self, limit: u64) -> Result<Self, GetAuditLogError> {
        if !validate::get_audit_log_limit(limit) {
            return Err(GetAuditLogError {
                kind: GetAuditLogErrorType::LimitInvalid,
            });
        }

        self.fields.limit = Some(limit);

        Ok(self)
    }

    /// Filter audit log for entries from a user.
    ///
    /// This is the user who did the auditable action, not the target of the auditable action.
    pub const fn user_id(mut self, user_id: UserId) -> Self {
        self.fields.user_id = Some(user_id);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<AuditLog> {
        let request = Request::from_route(Route::GetAuditLogs {
            action_type: self.fields.action_type.map(|x| x as u64),
            before: self.fields.before,
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            user_id: self.fields.user_id.map(|x| x.0),
        });

        self.http.request(request)
    }
}
