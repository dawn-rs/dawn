use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{GuildId, UserId};

/// Remove a ban from a user in a guild.
///
/// # Examples
///
/// Unban user `200` from guild `100`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::{GuildId, UserId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = GuildId(100);
/// let user_id = UserId(200);
///
/// client.delete_ban(guild_id, user_id).exec().await?;
/// # Ok(()) }
/// ```
pub struct DeleteBan<'a> {
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
    reason: Option<&'a str>,
}

impl<'a> DeleteBan<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            guild_id,
            http,
            user_id,
            reason: None,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::DeleteBan {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        });

        if let Some(reason) = self.reason.as_ref() {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason<'a> for DeleteBan<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
