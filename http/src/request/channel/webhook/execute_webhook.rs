use super::ExecuteWebhookAndWait;
use crate::{
    client::Client,
    error::Error,
    request::{Form, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{embed::Embed, message::AllowedMentions},
    id::WebhookId,
};

#[derive(Default, Serialize)]
pub(crate) struct ExecuteWebhookFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_mentions: Option<AllowedMentions>,
}

/// Execute a webhook, sending a message to its channel.
///
/// You can only specify one of [`content`], [`embeds`], or [`file`].
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::WebhookId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
/// let id = WebhookId(432);
///
/// client
///     .execute_webhook(id, "webhook token")
///     .content("Pinkie...")
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`content`]: Self::content
/// [`embeds`]: Self::embeds
/// [`file`]: Self::file
pub struct ExecuteWebhook<'a> {
    pub(crate) fields: ExecuteWebhookFields,
    files: Vec<(String, Vec<u8>)>,
    pub(super) http: &'a Client,
    token: String,
    webhook_id: WebhookId,
}

impl<'a> ExecuteWebhook<'a> {
    pub(crate) fn new(http: &'a Client, webhook_id: WebhookId, token: impl Into<String>) -> Self {
        Self {
            fields: ExecuteWebhookFields::default(),
            files: Vec::new(),
            http,
            token: token.into(),
            webhook_id,
        }
    }

    /// Specify the [`AllowedMentions`] for the webhook message.
    pub fn allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed_mentions);

        self
    }

    /// The URL of the avatar of the webhook.
    pub fn avatar_url(mut self, avatar_url: impl Into<String>) -> Self {
        self.fields.avatar_url.replace(avatar_url.into());

        self
    }

    /// The content of the webook's message.
    ///
    /// Up to 2000 UTF-16 codepoints, same as a message.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.fields.content.replace(content.into());

        self
    }

    /// Set the list of embeds of the webhook's message.
    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.fields.embeds.replace(embeds);

        self
    }

    /// Attach a file to the webhook.
    ///
    /// This method is repeatable.
    pub fn file(mut self, name: impl Into<String>, file: impl Into<Vec<u8>>) -> Self {
        self.files.push((name.into(), file.into()));

        self
    }

    /// Attach multiple files to the webhook.
    pub fn files<N: Into<String>, F: Into<Vec<u8>>>(
        mut self,
        attachments: impl IntoIterator<Item = (N, F)>,
    ) -> Self {
        for (name, file) in attachments {
            self = self.file(name, file);
        }

        self
    }

    /// JSON encoded body of any additional request fields.
    ///
    /// If this method is called, all other fields are ignored, except for
    /// [`file`]. See [Discord Docs/Create Message].
    ///
    /// # Examples
    ///
    /// Without [`payload_json`]:
    ///
    /// ```rust,no_run
    /// use twilight_embed_builder::EmbedBuilder;
    /// # use twilight_http::Client;
    /// use twilight_model::id::{MessageId, WebhookId};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// let message = client.execute_webhook(WebhookId(1), "token here")
    ///     .content("some content")
    ///     .embeds(vec![EmbedBuilder::new().title("title").build()?])
    ///     .wait()
    ///     .exec()
    ///     .await?
    ///     .model()
    ///     .await?;
    ///
    /// assert_eq!(message.content, "some content");
    /// # Ok(()) }
    /// ```
    ///
    /// With [`payload_json`]:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::{MessageId, WebhookId};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// let message = client.execute_webhook(WebhookId(1), "token here")
    ///     .content("some content")
    ///     .payload_json(r#"{ "content": "other content", "embeds": [ { "title": "title" } ] }"#)
    ///     .wait()
    ///     .exec()
    ///     .await?
    ///     .model()
    ///     .await?;
    ///
    /// assert_eq!(message.content, "other content");
    /// # Ok(()) }
    /// ```
    ///
    /// [`payload_json`]: Self::payload_json
    /// [Discord Docs/Create Message]: https://discord.com/developers/docs/resources/channel#create-message-params
    pub fn payload_json(mut self, payload_json: impl Into<Vec<u8>>) -> Self {
        self.fields.payload_json.replace(payload_json.into());

        self
    }

    /// Specify true if the message is TTS.
    pub fn tts(mut self, tts: bool) -> Self {
        self.fields.tts.replace(tts);

        self
    }

    /// Specify the username of the webhook's message.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.fields.username.replace(username.into());

        self
    }

    /// Wait for the message to send before sending a response. See
    /// [Discord Docs/Execute Webhook].
    ///
    /// Using this will result in receiving the created message.
    ///
    /// [Discord Docs/Execute Webhook]: https://discord.com/developers/docs/resources/webhook#execute-webhook-querystring-params
    #[allow(clippy::missing_const_for_fn)]
    pub fn wait(self) -> ExecuteWebhookAndWait<'a> {
        ExecuteWebhookAndWait::new(self)
    }

    pub(super) fn request(self, wait: bool) -> Result<(Request, &'a Client), Error> {
        let mut request = Request::builder(Route::ExecuteWebhook {
            token: self.token.clone(),
            wait: Some(wait),
            webhook_id: self.webhook_id.0,
        });

        // Webhook executions don't need the authorization token, only the
        // webhook token.
        request = request.use_authorization_token(false);

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.iter().enumerate() {
                form.file(format!("{}", index).as_bytes(), name.as_bytes(), file);
            }

            if let Some(payload_json) = &self.fields.payload_json {
                form.payload_json(&payload_json);
            } else {
                let body = crate::json::to_vec(&self.fields).map_err(Error::json)?;

                form.payload_json(&body);
            }

            request = request.form(form);
        } else {
            request = request.json(&self.fields)?;
        }

        Ok((request.build(), self.http))
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let (request, client) = match self.request(false) {
            Ok((request, client)) => (request, client),
            Err(source) => return ResponseFuture::error(source),
        };

        client.request(request)
    }
}
