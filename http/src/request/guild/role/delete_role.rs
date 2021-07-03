use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{GuildId, RoleId};

/// Delete a role in a guild, by id.
pub struct DeleteRole<'a> {
    guild_id: GuildId,
    http: &'a Client,
    role_id: RoleId,
    reason: Option<&'a str>,
}

impl<'a> DeleteRole<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, role_id: RoleId) -> Self {
        Self {
            guild_id,
            http,
            role_id,
            reason: None,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::DeleteRole {
            guild_id: self.guild_id.0,
            role_id: self.role_id.0,
        });

        if let Some(reason) = &self.reason {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason<'a> for DeleteRole<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
