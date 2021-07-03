use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, NullableField, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::Webhook,
    id::{ChannelId, WebhookId},
};

#[derive(Default, Serialize)]
struct UpdateWebhookFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<NullableField<String>>,
}

/// Update a webhook by ID.
pub struct UpdateWebhook<'a> {
    fields: UpdateWebhookFields,
    http: &'a Client,
    webhook_id: WebhookId,
    reason: Option<String>,
}

/// Update a webhook by its ID.
impl<'a> UpdateWebhook<'a> {
    pub(crate) fn new(http: &'a Client, webhook_id: WebhookId) -> Self {
        Self {
            fields: UpdateWebhookFields::default(),
            http,
            webhook_id,
            reason: None,
        }
    }

    /// Set the avatar of the webhook.
    ///
    /// See [Discord Docs/Image Data] for more information. This must be a Data URI, in the form of
    /// `data:image/{type};base64,{data}` where `{type}` is the image MIME type and `{data}` is the
    /// base64-encoded image.
    ///
    /// [Discord Docs/Image Data]: https://discord.com/developers/docs/reference#image-data
    pub fn avatar(mut self, avatar: impl Into<Option<String>>) -> Self {
        self.fields
            .avatar
            .replace(NullableField::from_option(avatar.into()));

        self
    }

    /// Move this webhook to a new channel.
    pub fn channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.fields.channel_id.replace(channel_id.into());

        self
    }

    /// Change the name of the webhook.
    pub fn name(mut self, name: impl Into<Option<String>>) -> Self {
        self.fields
            .name
            .replace(NullableField::from_option(name.into()));

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Webhook> {
        let mut request = Request::builder(Route::UpdateWebhook {
            token: None,
            webhook_id: self.webhook_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

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

impl<'a> AuditLogReason for UpdateWebhook<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
