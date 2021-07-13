use crate::{
    client::Client,
    error::{Error as HttpError, ErrorType},
    request::{
        validate_inner::{self, ComponentValidationError, ComponentValidationErrorType},
        Form, PendingOption, Request,
    },
    routing::Route,
};
use hyper::StatusCode;
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_model::{
    application::component::Component,
    channel::{embed::Embed, message::AllowedMentions, Message},
    id::WebhookId,
};

/// A webhook could not be executed.
#[derive(Debug)]
pub struct ExecuteWebhookError {
    kind: ExecuteWebhookErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl ExecuteWebhookError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ExecuteWebhookErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        ExecuteWebhookErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for ExecuteWebhookError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ExecuteWebhookErrorType::ComponentCount { count } => {
                Display::fmt(count, f)?;
                f.write_str(" components were provided, but only ")?;
                Display::fmt(&ComponentValidationError::COMPONENT_COUNT, f)?;

                f.write_str(" root components are allowed")
            }
            ExecuteWebhookErrorType::ComponentInvalid { .. } => {
                f.write_str("a provided component is invalid")
            }
        }
    }
}

impl Error for ExecuteWebhookError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`ExecuteWebhookError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExecuteWebhookErrorType {
    /// Too many message components were provided.
    ComponentCount {
        /// Number of components that were provided.
        count: usize,
    },
    /// An invalid message component was provided.
    ComponentInvalid {
        /// Additional details about the validation failure type.
        kind: ComponentValidationErrorType,
    },
}

#[derive(Default, Serialize)]
pub(crate) struct ExecuteWebhookFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_mentions: Option<AllowedMentions>,
}

/// Executes a webhook, sending a message to its channel.
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
/// let webhook = client
///     .execute_webhook(id, "webhook token")
///     .content("Pinkie...")
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
    fut: Option<PendingOption<'a>>,
    http: &'a Client,
    token: String,
    webhook_id: WebhookId,
}

impl<'a> ExecuteWebhook<'a> {
    pub(crate) fn new(http: &'a Client, webhook_id: WebhookId, token: impl Into<String>) -> Self {
        Self {
            fields: ExecuteWebhookFields::default(),
            files: Vec::new(),
            fut: None,
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

    /// Add multiple [`Component`]s to a message.
    ///
    /// Calling this method multiple times will clear previous calls.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecuteWebhookErrorType::ComponentCount`] error
    /// type if too many components are provided.
    ///
    /// Returns an [`ExecuteWebhookErrorType::ComponentInvalid`] error
    /// type if one of the provided components is invalid.
    pub fn components(mut self, components: Vec<Component>) -> Result<Self, ExecuteWebhookError> {
        validate_inner::components(&components).map_err(|source| {
            let (kind, inner_source) = source.into_parts();

            match kind {
                ComponentValidationErrorType::ComponentCount { count } => ExecuteWebhookError {
                    kind: ExecuteWebhookErrorType::ComponentCount { count },
                    source: inner_source,
                },
                other => ExecuteWebhookError {
                    kind: ExecuteWebhookErrorType::ComponentInvalid { kind: other },
                    source: inner_source,
                },
            }
        })?;

        self.fields.components = components;

        Ok(self)
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
    ///     .wait(true)
    ///     .await?
    ///     .unwrap();
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
    ///     .wait(true)
    ///     .await?
    ///     .unwrap();
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

    /// If true, wait for the message to send before sending a response. See [Discord Docs/Execute
    /// Webhook]
    ///
    /// [Discord Docs/Execute Webhook]: https://discord.com/developers/docs/resources/webhook#execute-webhook-querystring-params
    pub fn wait(mut self, wait: bool) -> Self {
        self.fields.wait.replace(wait);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let mut request = Request::builder(Route::ExecuteWebhook {
            token: self.token.clone(),
            wait: self.fields.wait,
            webhook_id: self.webhook_id.0,
        });

        // Webhook executions don't need the authorization token, only the
        // webhook token.
        request = request.use_authorization_token(false);

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.drain(..).enumerate() {
                form.file(format!("{}", index).as_bytes(), name.as_bytes(), &file);
            }

            if let Some(payload_json) = &self.fields.payload_json {
                form.payload_json(&payload_json);
            } else {
                let body = crate::json::to_vec(&self.fields).map_err(HttpError::json)?;
                form.payload_json(&body);
            }

            request = request.form(form);
        } else {
            request = request.json(&self.fields)?;
        }

        self.fut
            .replace(Box::pin(self.http.request_bytes(request.build())));

        Ok(())
    }
}

impl Future for ExecuteWebhook<'_> {
    type Output = Result<Option<Message>, HttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(fut) = self.as_mut().fut.as_mut() {
                let bytes = match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(bytes)) => bytes,
                    Poll::Ready(Err(HttpError {
                        kind: ErrorType::Response { status, .. },
                        source: None,
                    })) if status == StatusCode::NOT_FOUND => {
                        return Poll::Ready(Ok(None));
                    }
                    Poll::Ready(Err(why)) => return Poll::Ready(Err(why)),
                    Poll::Pending => return Poll::Pending,
                };

                if !self.fields.wait.unwrap_or_default() {
                    return Poll::Ready(Ok(None));
                }

                let message = crate::json::parse_bytes::<Message>(&bytes)?;

                return Poll::Ready(Ok(Some(message)));
            }

            if let Err(why) = self.as_mut().start() {
                return Poll::Ready(Err(why));
            }
        }
    }
}
