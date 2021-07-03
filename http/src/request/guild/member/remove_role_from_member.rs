use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{GuildId, RoleId, UserId};

/// Remove a role from a member in a guild, by id.
pub struct RemoveRoleFromMember<'a> {
    guild_id: GuildId,
    http: &'a Client,
    role_id: RoleId,
    user_id: UserId,
    reason: Option<String>,
}

impl<'a> RemoveRoleFromMember<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: impl Into<GuildId>,
        user_id: impl Into<UserId>,
        role_id: impl Into<RoleId>,
    ) -> Self {
        Self {
            guild_id: guild_id.into(),
            http,
            role_id: role_id.into(),
            user_id: user_id.into(),
            reason: None,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::RemoveMemberRole {
            guild_id: self.guild_id.0,
            role_id: self.role_id.0,
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

impl<'a> AuditLogReason for RemoveRoleFromMember<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
