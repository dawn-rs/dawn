use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{channel::Webhook, id::ChannelId};

#[derive(Serialize)]
struct CreateWebhookFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<&'a str>,
    name: &'a str,
}

/// Create a webhook in a channel.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::ChannelId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
/// let channel_id = ChannelId(123);
///
/// let webhook = client
///     .create_webhook(channel_id, "Twily Bot")
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct CreateWebhook<'a> {
    channel_id: ChannelId,
    fields: CreateWebhookFields<'a>,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> CreateWebhook<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId, name: &'a str) -> Self {
        Self {
            channel_id,
            fields: CreateWebhookFields { avatar: None, name },
            http,
            reason: None,
        }
    }

    /// Set the avatar of the webhook.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn avatar(mut self, avatar: &'a str) -> Self {
        self.fields.avatar.replace(avatar);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Webhook> {
        let mut request = Request::builder(Route::CreateWebhook {
            channel_id: self.channel_id.0,
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

impl<'a> AuditLogReason<'a> for CreateWebhook<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
