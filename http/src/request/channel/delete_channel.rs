use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::ResponseFuture,
    routing::Route,
};
use twilight_model::{channel::Channel, id::ChannelId};

/// Delete a channel by ID.
pub struct DeleteChannel<'a> {
    channel_id: ChannelId,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> DeleteChannel<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            http,
            reason: None,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Channel> {
        let mut request = Request::builder(Route::DeleteChannel {
            channel_id: self.channel_id.0,
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

impl<'a> AuditLogReason<'a> for DeleteChannel<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
