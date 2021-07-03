use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        multipart::Form,
        validate::{self, EmbedValidationError},
        Request,
    },
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::{
        embed::Embed,
        message::{AllowedMentions, MessageReference},
        Message,
    },
    id::{ChannelId, MessageId},
};

/// The error created when a message can not be created as configured.
#[derive(Debug)]
pub struct CreateMessageError {
    kind: CreateMessageErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CreateMessageError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateMessageErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (CreateMessageErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }

    fn embed(source: EmbedValidationError, embed: Embed, idx: Option<usize>) -> Self {
        Self {
            kind: CreateMessageErrorType::EmbedTooLarge {
                embed: Box::new(embed),
                idx,
            },
            source: Some(Box::new(source)),
        }
    }
}

impl Display for CreateMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateMessageErrorType::ContentInvalid { .. } => {
                f.write_str("the message content is invalid")
            }
            CreateMessageErrorType::EmbedTooLarge { idx, .. } => {
                if let Some(idx) = idx {
                    f.write_str("the embed at index ")?;
                    Display::fmt(&idx, f)?;

                    f.write_str("'s contents are too long")
                } else {
                    f.write_str("the embed's contents are too long")
                }
            }
        }
    }
}

impl Error for CreateMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`CreateMessageError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateMessageErrorType {
    /// Returned when the content is over 2000 UTF-16 characters.
    ContentInvalid {
        /// Provided content.
        content: String,
    },
    /// Returned when the length of the embed is over 6000 characters.
    EmbedTooLarge {
        /// Provided embed.
        embed: Box<Embed>,
        /// Index of the embed, if there is any.
        idx: Option<usize>,
    },
}

#[derive(Default, Serialize)]
pub(crate) struct CreateMessageFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    embeds: Vec<Embed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
}

/// Send a message to a channel.
///
/// # Example
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::ChannelId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
///
/// let channel_id = ChannelId(123);
/// let message = client
///     .create_message(channel_id)
///     .content("Twilight is best pony")?
///     .tts(true)
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct CreateMessage<'a> {
    channel_id: ChannelId,
    pub(crate) fields: CreateMessageFields,
    files: Vec<(String, Vec<u8>)>,
    http: &'a Client,
}

impl<'a> CreateMessage<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: CreateMessageFields {
                allowed_mentions: http.default_allowed_mentions(),
                ..CreateMessageFields::default()
            },
            files: Vec::new(),
            http,
        }
    }

    /// Specify the [`AllowedMentions`] for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed_mentions);

        self
    }

    /// Set the content of the message.
    ///
    /// The maximum length is 2000 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateMessageErrorType::ContentInvalid`] error type if the
    /// content length is too long.
    pub fn content(self, content: impl Into<String>) -> Result<Self, CreateMessageError> {
        self._content(content.into())
    }

    fn _content(mut self, content: String) -> Result<Self, CreateMessageError> {
        if !validate::content_limit(&content) {
            return Err(CreateMessageError {
                kind: CreateMessageErrorType::ContentInvalid { content },
                source: None,
            });
        }

        self.fields.content.replace(content);

        Ok(self)
    }

    /// Attach an embed to the message.
    ///
    /// Embed total character length must not exceed 6000 characters.
    /// Additionally, the internal fields also have character limits. Refer to
    /// [the discord docs] for more information.
    ///
    /// # Examples
    ///
    /// See [`EmbedBuilder`] for an example.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateMessageErrorType::EmbedTooLarge`] error type if the
    /// embed is too large.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#embed-limits
    /// [`EmbedBuilder`]: https://docs.rs/twilight-embed-builder/*/twilight_embed_builder
    pub fn embed(mut self, embed: Embed) -> Result<Self, CreateMessageError> {
        validate::embed(&embed)
            .map_err(|source| CreateMessageError::embed(source, embed.clone(), None))?;

        self.fields.embeds.push(embed);

        Ok(self)
    }

    /// Attach multiple embeds to the message.
    ///
    /// Embed total character length must not exceed 6000 characters.
    /// Additionally, the internal fields also have character limits. Refer to
    /// [the discord docs] for more information.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateMessageErrorType::EmbedTooLarge`] error type if an
    /// embed is too large.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#embed-limits
    pub fn embeds(
        mut self,
        embeds: impl IntoIterator<Item = Embed>,
    ) -> Result<Self, CreateMessageError> {
        for (idx, embed) in embeds.into_iter().enumerate() {
            validate::embed(&embed)
                .map_err(|source| CreateMessageError::embed(source, embed.clone(), Some(idx)))?;

            self.fields.embeds.push(embed);
        }

        Ok(self)
    }

    /// Whether to fail sending if the reply no longer exists.
    pub fn fail_if_not_exists(mut self) -> Self {
        self.fields.message_reference = Some(self.fields.message_reference.map_or_else(
            || MessageReference {
                channel_id: None,
                guild_id: None,
                message_id: None,
                fail_if_not_exists: Some(true),
            },
            |message_reference| MessageReference {
                fail_if_not_exists: Some(true),
                ..message_reference
            },
        ));

        self
    }

    /// Attach a file to the message.
    ///
    /// The file is raw binary data. It can be an image, or any other kind of file.
    pub fn file(mut self, name: impl Into<String>, file: impl Into<Vec<u8>>) -> Self {
        self.files.push((name.into(), file.into()));

        self
    }

    /// Attach multiple files to the message.
    pub fn files<N: Into<String>, F: Into<Vec<u8>>>(
        mut self,
        attachments: impl IntoIterator<Item = (N, F)>,
    ) -> Self {
        for (name, file) in attachments {
            self = self.file(name, file);
        }

        self
    }

    /// Attach a nonce to the message, for optimistic message sending.
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.fields.nonce.replace(nonce);

        self
    }

    /// JSON encoded body of any additional request fields.
    ///
    /// If this method is called, all other fields are ignored, except for
    /// [`file`]. See [Discord Docs/Create Message].
    ///
    /// [`file`]: Self::file
    /// [Discord Docs/Create Message]: https://discord.com/developers/docs/resources/channel#create-message-params
    pub fn payload_json(mut self, payload_json: impl Into<Vec<u8>>) -> Self {
        self.fields.payload_json.replace(payload_json.into());

        self
    }

    /// Specify the ID of another message to create a reply to.
    pub fn reply(mut self, other: MessageId) -> Self {
        let channel_id = self.channel_id;

        self.fields.message_reference = Some(self.fields.message_reference.map_or_else(
            || MessageReference {
                channel_id: Some(channel_id),
                guild_id: None,
                message_id: Some(other),
                fail_if_not_exists: None,
            },
            |message_reference| MessageReference {
                channel_id: Some(channel_id),
                message_id: Some(other),
                ..message_reference
            },
        ));

        self
    }

    /// Specify true if the message is TTS.
    pub fn tts(mut self, tts: bool) -> Self {
        self.fields.tts.replace(tts);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Message> {
        let mut request = Request::builder(Route::CreateMessage {
            channel_id: self.channel_id.0,
        });

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.iter().enumerate() {
                form.file(format!("{}", index).as_bytes(), name.as_bytes(), file);
            }

            if let Some(payload_json) = &self.fields.payload_json {
                form.payload_json(&payload_json);
            } else {
                let body = match crate::json::to_vec(&self.fields) {
                    Ok(body) => body,
                    Err(source) => return ResponseFuture::error(HttpError::json(source)),
                };

                form.payload_json(&body);
            }

            request = request.form(form);
        } else {
            request = match request.json(&self.fields) {
                Ok(request) => request,
                Err(source) => return ResponseFuture::error(source),
            };
        }

        self.http.request(request.build())
    }
}
