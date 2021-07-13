use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        validate::{ComponentValidationError, ComponentValidationErrorType},
        validate_inner, Form, Pending, Request,
    },
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    application::component::Component,
    channel::{
        embed::Embed,
        message::{AllowedMentions, MessageFlags},
        Message,
    },
    id::ApplicationId,
};

/// A followup message can not be created as configured.
#[derive(Debug)]
pub struct CreateFollowupMessageError {
    kind: CreateFollowupMessageErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CreateFollowupMessageError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateFollowupMessageErrorType {
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
        CreateFollowupMessageErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for CreateFollowupMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateFollowupMessageErrorType::ComponentCount { count } => {
                Display::fmt(count, f)?;
                f.write_str(" components were provided, but only ")?;
                Display::fmt(&ComponentValidationError::COMPONENT_COUNT, f)?;

                f.write_str(" root components are allowed")
            }
            CreateFollowupMessageErrorType::ComponentInvalid { .. } => {
                f.write_str("a provided component is invalid")
            }
        }
    }
}

impl Error for CreateFollowupMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`CreateFollowupMessageError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateFollowupMessageErrorType {
    /// An invalid message component was provided.
    ComponentInvalid {
        /// Additional details about the validation failure type.
        kind: ComponentValidationErrorType,
    },
    /// Too many message components were provided.
    ComponentCount {
        /// Number of components that were provided.
        count: usize,
    },
}

#[derive(Default, Serialize)]
pub(crate) struct CreateFollowupMessageFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<Component>,
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
    flags: Option<MessageFlags>,
    allowed_mentions: Option<AllowedMentions>,
}

/// Create a followup message to an interaction.
///
/// You must specify at least one of [`content`], [`embeds`], or [`file`].
///
/// # Examples
///
/// ```rust,no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
/// use twilight_model::id::ApplicationId;
///
/// let client = Client::new(env::var("DISCORD_TOKEN")?);
/// client.set_application_id(ApplicationId(1));
///
/// let webhook = client
///     .create_followup_message("webhook token")?
///     .content("Pinkie...")
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`content`]: Self::content
/// [`embeds`]: Self::embeds
/// [`file`]: Self::file
pub struct CreateFollowupMessage<'a> {
    pub(crate) fields: CreateFollowupMessageFields,
    files: Vec<(String, Vec<u8>)>,
    fut: Option<Pending<'a, Option<Message>>>,
    http: &'a Client,
    token: String,
    application_id: ApplicationId,
}

impl<'a> CreateFollowupMessage<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        token: impl Into<String>,
    ) -> Self {
        Self {
            fields: CreateFollowupMessageFields::default(),
            files: Vec::new(),
            fut: None,
            http,
            token: token.into(),
            application_id,
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
    /// Returns an [`CreateFollowupMessageErrorType::ComponentCount`] error type
    /// if too many components are provided.
    ///
    /// Returns an [`CreateFollowupMessageErrorType::ComponentInvalid`] error
    /// type if one of the provided components is invalid.
    pub fn components(
        mut self,
        components: Vec<Component>,
    ) -> Result<Self, CreateFollowupMessageError> {
        validate_inner::components(&components).map_err(|source| {
            let (kind, inner_source) = source.into_parts();

            match kind {
                ComponentValidationErrorType::ComponentCount { count } => {
                    CreateFollowupMessageError {
                        kind: CreateFollowupMessageErrorType::ComponentCount { count },
                        source: inner_source,
                    }
                }
                other => CreateFollowupMessageError {
                    kind: CreateFollowupMessageErrorType::ComponentInvalid { kind: other },
                    source: inner_source,
                },
            }
        })?;

        self.fields.components = components;

        Ok(self)
    }

    /// The content of the webook's message.
    ///
    /// Up to 2000 UTF-16 codepoints.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.fields.content.replace(content.into());

        self
    }

    /// Set the list of embeds of the webhook's message.
    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.fields.embeds.replace(embeds);

        self
    }

    /// Set if the followup should be ephemeral.
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        if ephemeral {
            self.fields.flags.replace(MessageFlags::EPHEMERAL);
        } else {
            self.fields.flags = None;
        }

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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env;
    /// use twilight_embed_builder::EmbedBuilder;
    /// use twilight_http::Client;
    /// use twilight_model::id::{MessageId, ApplicationId};
    ///
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// client.set_application_id(ApplicationId(1));
    ///
    /// let message = client.create_followup_message("token here")?
    ///     .content("some content")
    ///     .embeds(vec![EmbedBuilder::new().title("title").build()?])
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
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env;
    /// use twilight_http::Client;
    /// use twilight_model::id::{MessageId, ApplicationId};
    ///
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// client.set_application_id(ApplicationId(1));
    ///
    /// let message = client.create_followup_message("token here")?
    ///     .content("some content")
    ///     .payload_json(r#"{ "content": "other content", "embeds": [ { "title": "title" } ] }"#)
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

    fn start(&mut self) -> Result<(), HttpError> {
        let mut request = Request::builder(Route::ExecuteWebhook {
            token: self.token.clone(),
            wait: None,
            webhook_id: self.application_id.0,
        });

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
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

poll_req!(CreateFollowupMessage<'_>, Option<Message>);
