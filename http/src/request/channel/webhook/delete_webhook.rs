use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::WebhookId;

struct DeleteWebhookParams {
    token: Option<String>,
}

/// Delete a webhook by its ID.
pub struct DeleteWebhook<'a> {
    fields: DeleteWebhookParams,
    http: &'a Client,
    id: WebhookId,
    reason: Option<String>,
}

impl<'a> DeleteWebhook<'a> {
    pub(crate) const fn new(http: &'a Client, id: WebhookId) -> Self {
        Self {
            fields: DeleteWebhookParams { token: None },
            http,
            id,
            reason: None,
        }
    }

    /// Specify the token for auth, if not already authenticated with a Bot token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.fields.token.replace(token.into());

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::DeleteWebhook {
            webhook_id: self.id.0,
            token: self.fields.token,
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

impl<'a> AuditLogReason for DeleteWebhook<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
