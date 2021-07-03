use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::ChannelId;

/// Clear the permissions for a target ID in a channel.
///
/// The `target_id` is a `u64`, but it should point to a `RoleId` or a `UserId`.
pub struct DeleteChannelPermissionConfigured<'a> {
    channel_id: ChannelId,
    http: &'a Client,
    reason: Option<String>,
    target_id: u64,
}

impl<'a> DeleteChannelPermissionConfigured<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId, target_id: u64) -> Self {
        Self {
            channel_id,
            http,
            reason: None,
            target_id,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::DeletePermissionOverwrite {
            channel_id: self.channel_id.0,
            target_id: self.target_id,
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

impl<'a> AuditLogReason for DeleteChannelPermissionConfigured<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
