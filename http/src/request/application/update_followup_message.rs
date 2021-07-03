//! Update a followup message created from a interaction.

use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, Form, NullableField, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::{embed::Embed, message::AllowedMentions, Attachment},
    id::{ApplicationId, MessageId},
};

/// A followup message can not be updated as configured.
#[derive(Debug)]
pub struct UpdateFollowupMessageError {
    kind: UpdateFollowupMessageErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl UpdateFollowupMessageError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateFollowupMessageErrorType {
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
        UpdateFollowupMessageErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for UpdateFollowupMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateFollowupMessageErrorType::ContentInvalid => {
                f.write_str("message content is invalid")
            }
            UpdateFollowupMessageErrorType::EmbedTooLarge { .. } => {
                f.write_str("length of one of the embeds is too large")
            }
            UpdateFollowupMessageErrorType::TooManyEmbeds => {
                f.write_str("only 10 embeds may be provided")
            }
        }
    }
}

impl Error for UpdateFollowupMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`UpdateFollowupMessageError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateFollowupMessageErrorType {
    /// Content is over 2000 UTF-16 characters.
    ContentInvalid,
    /// Length of one of the embeds is over 6000 characters.
    EmbedTooLarge {
        /// Index of the embed that was too large.
        ///
        /// This can be used to index into the provided embeds to retrieve the
        /// invalid embed.
        ///
        /// [`embeds`]: Self::EmbedTooLarge.embeds
        index: usize,
    },
    /// Too many embeds were provided.
    ///
    /// A followup message can have up to 10 embeds.
    TooManyEmbeds,
}

#[derive(Default, Serialize)]
struct UpdateFollowupMessageFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "request::slice_is_empty")]
    attachments: &'a [Attachment],
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<NullableField<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<NullableField<&'a [Embed]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<&'a [u8]>,
}

/// Update a followup message.
///
/// A followup message must always have at least one embed or some amount of
/// content. If you wish to delete a followup message refer to
/// [`DeleteFollowupMessage`].
///
/// # Examples
///
/// Update a followup message by setting the content to `test <@3>` -
/// attempting to mention user ID 3 - and specifying that only that the user may
/// not be mentioned.
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
/// use twilight_model::{
///     channel::message::AllowedMentions,
///     id::{MessageId, ApplicationId}
/// };
///
/// let client = Client::new(env::var("DISCORD_TOKEN")?);
/// client.set_application_id(ApplicationId(1));
///
/// client.update_followup_message("token here", MessageId(2))?
///     // By creating a default set of allowed mentions, no entity can be
///     // mentioned.
///     .allowed_mentions(AllowedMentions::default())
///     .content(Some("test <@3>"))?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`DeleteFollowupMessage`]: super::DeleteFollowupMessage
pub struct UpdateFollowupMessage<'a> {
    fields: UpdateFollowupMessageFields<'a>,
    files: &'a [(&'a str, &'a [u8])],
    http: &'a Client,
    message_id: MessageId,
    token: &'a str,
    application_id: ApplicationId,
}

impl<'a> UpdateFollowupMessage<'a> {
    /// Maximum number of embeds that a followup message may have.
    pub const EMBED_COUNT_LIMIT: usize = 10;

    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        token: &'a str,
        message_id: MessageId,
    ) -> Self {
        Self {
            fields: UpdateFollowupMessageFields {
                allowed_mentions: http.default_allowed_mentions(),
                ..UpdateFollowupMessageFields::default()
            },
            files: &[],
            http,
            message_id,
            token,
            application_id,
        }
    }

    /// Set the allowed mentions in the message.
    pub fn allowed_mentions(mut self, allowed: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed);

        self
    }

    /// Specify multiple attachments already present in the target message to keep.
    ///
    /// If called, all unspecified attachments will be removed from the message.
    /// If not called, all attachments will be kept.
    pub const fn attachments(mut self, attachments: &'a [Attachment]) -> Self {
        self.fields.attachments = attachments;

        self
    }

    /// Set the content of the message.
    ///
    /// Pass `None` if you want to remove the message content.
    ///
    /// Note that if there is are no embeds then you will not be able to remove
    /// the content of the message.
    ///
    /// The maximum length is 2000 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateFollowupMessageErrorType::ContentInvalid`] error type if
    /// the content length is too long.
    pub fn content(mut self, content: Option<&'a str>) -> Result<Self, UpdateFollowupMessageError> {
        if let Some(content_ref) = content.as_ref() {
            if !validate::content_limit(content_ref) {
                return Err(UpdateFollowupMessageError {
                    kind: UpdateFollowupMessageErrorType::ContentInvalid,
                    source: None,
                });
            }
        }

        self.fields.content = Some(NullableField(content));

        Ok(self)
    }

    /// Set the list of embeds of the followup message.
    ///
    /// Pass `None` to remove all of the embeds.
    ///
    /// The maximum number of allowed embeds is defined by
    /// [`EMBED_COUNT_LIMIT`].
    ///
    /// The total character length of each embed must not exceed 6000
    /// characters. Additionally, the internal fields also have character
    /// limits. Refer to [the discord docs] for more information.
    ///
    /// # Examples
    ///
    /// Create an embed and update the message with the new embed. The content
    /// of the original message is unaffected and only the embed(s) are
    /// modified.
    ///
    /// ```no_run
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::env;
    /// use twilight_http::Client;
    /// use twilight_embed_builder::EmbedBuilder;
    /// use twilight_model::id::{ApplicationId, MessageId};
    ///
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// client.set_application_id(ApplicationId(1));
    ///
    /// let embed = EmbedBuilder::new()
    ///     .description("Powerful, flexible, and scalable ecosystem of Rust libraries for the Discord API.")
    ///     .title("Twilight")
    ///     .url("https://twilight.rs")
    ///     .build()?;
    ///
    /// client.update_followup_message("token", MessageId(2))?
    ///     .embeds(Some(&[embed]))?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateFollowupMessageErrorType::EmbedTooLarge`] error type
    /// if one of the embeds are too large.
    ///
    /// Returns an [`UpdateFollowupMessageErrorType::TooManyEmbeds`] error type
    /// if more than 10 embeds are provided.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#embed-limits
    /// [`EMBED_COUNT_LIMIT`]: Self::EMBED_COUNT_LIMIT
    pub fn embeds(
        mut self,
        embeds: Option<&'a [Embed]>,
    ) -> Result<Self, UpdateFollowupMessageError> {
        if let Some(embeds_present) = embeds.as_deref() {
            if embeds_present.len() > Self::EMBED_COUNT_LIMIT {
                return Err(UpdateFollowupMessageError {
                    kind: UpdateFollowupMessageErrorType::TooManyEmbeds,
                    source: None,
                });
            }

            for (idx, embed) in embeds_present.iter().enumerate() {
                if let Err(source) = validate::embed(embed) {
                    return Err(UpdateFollowupMessageError {
                        kind: UpdateFollowupMessageErrorType::EmbedTooLarge { index: idx },
                        source: Some(Box::new(source)),
                    });
                }
            }
        }

        self.fields.embeds = Some(NullableField(embeds));

        Ok(self)
    }

    /// Attach multiple files to the followup message.
    pub const fn files(mut self, files: &'a [(&'a str, &'a [u8])]) -> Self {
        self.files = files;

        self
    }

    /// JSON encoded body of any additional request fields.
    ///
    /// If this method is called, all other fields are ignored, except for
    /// [`files`]. See [Discord Docs/Create Message] and
    /// [`CreateFollowupMessage::payload_json`].
    ///
    /// [`files`]: Self::files
    /// [`CreateFollowupMessage::payload_json`]: super::CreateFollowupMessage::payload_json
    /// [Discord Docs/Create Message]: https://discord.com/developers/docs/resources/channel#create-message-params
    pub const fn payload_json(mut self, payload_json: &'a [u8]) -> Self {
        self.fields.payload_json = Some(payload_json);

        self
    }

    fn request(&mut self) -> Result<Request<'a>, HttpError> {
        let mut request = Request::builder(Route::UpdateWebhookMessage {
            message_id: self.message_id.0,
            token: self.token,
            webhook_id: self.application_id.0,
        });

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.iter().enumerate() {
                form.file(index.to_be_bytes().as_ref(), name.as_bytes(), file);
            }

            if let Some(payload_json) = &self.fields.payload_json {
                form.payload_json(payload_json);
            } else {
                let body = crate::json::to_vec(&self.fields).map_err(HttpError::json)?;
                form.payload_json(&body);
            }

            request = request.form(form);
        } else {
            request = request.json(&self.fields)?;
        }

        Ok(request.build())
    }

    pub fn exec(mut self) -> ResponseFuture<EmptyBody> {
        match self.request() {
            Ok(request) => self.http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}
