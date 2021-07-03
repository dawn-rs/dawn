use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use twilight_model::id::{ChannelId, MessageId};

#[derive(Serialize)]
struct DeleteMessagesFields<'a> {
    messages: &'a [MessageId],
}

/// Delete messgaes by [`ChannelId`] and a list of [`MessageId`]s.
///
/// The number of message IDs must be between 2 and 100. If the supplied message
/// IDs are invalid, they still count towards the lower and upper limits. This
/// method will not delete messages older than two weeks. Refer to
/// [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/channel#bulk-delete-messages
pub struct DeleteMessages<'a> {
    channel_id: ChannelId,
    fields: DeleteMessagesFields<'a>,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> DeleteMessages<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        messages: &'a [MessageId],
    ) -> Self {
        Self {
            channel_id,
            fields: DeleteMessagesFields { messages },
            http,
            reason: None,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::DeleteMessages {
            channel_id: self.channel_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

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

impl<'a> AuditLogReason<'a> for DeleteMessages<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
