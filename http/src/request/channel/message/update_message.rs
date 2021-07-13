use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        validate_inner::{
            self, ComponentValidationError, ComponentValidationErrorType, EmbedValidationError,
        },
        NullableField, Pending, Request,
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
        Attachment, Message,
    },
    id::{ChannelId, MessageId},
};

/// The error created when a message can not be updated as configured.
#[derive(Debug)]
pub struct UpdateMessageError {
    kind: UpdateMessageErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl UpdateMessageError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateMessageErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (UpdateMessageErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }

    fn embed(source: EmbedValidationError, embed: Embed, idx: Option<usize>) -> Self {
        Self {
            kind: UpdateMessageErrorType::EmbedTooLarge {
                embed: Box::new(embed),
                idx,
            },
            source: Some(Box::new(source)),
        }
    }
}

impl Display for UpdateMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateMessageErrorType::ComponentCount { count } => {
                Display::fmt(count, f)?;
                f.write_str(" components were provided, but only ")?;
                Display::fmt(&ComponentValidationError::COMPONENT_COUNT, f)?;

                f.write_str(" root components are allowed")
            }
            UpdateMessageErrorType::ComponentInvalid { .. } => {
                f.write_str("a provided component is invalid")
            }
            UpdateMessageErrorType::ContentInvalid { .. } => {
                f.write_str("the message content is invalid")
            }
            UpdateMessageErrorType::EmbedTooLarge { idx, .. } => {
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

impl Error for UpdateMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`UpdateMessageError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateMessageErrorType {
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
struct UpdateMessageFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<NullableField<Vec<Component>>>,
    // We don't serialize if this is Option::None, to avoid overwriting the
    // field without meaning to.
    //
    // So we use a nested Option, representing the following states:
    //
    // - Some(Some(String)): Modifying the "content" from one state to a string;
    // - Some(None): Removing the "content" by giving the Discord API a written
    //   `"content": null` in the JSON;
    // - None: Don't serialize the field at all, not modifying the state.
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
}

/// Update a message by [`ChannelId`] and [`MessageId`].
///
/// You can pass `None` to any of the methods to remove the associated field.
/// For example, if you have a message with an embed you want to remove, you can
/// use `.embed(None)` to remove the embed.
///
/// # Examples
///
/// Replace the content with `"test update"`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::{ChannelId, MessageId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
/// client.update_message(ChannelId(1), MessageId(2))
///     .content("test update".to_owned())?
///     .await?;
/// # Ok(()) }
/// ```
///
/// Remove the message's content:
///
/// ```rust,no_run
/// # use twilight_http::Client;
/// # use twilight_model::id::{ChannelId, MessageId};
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = Client::new("my token");
/// client.update_message(ChannelId(1), MessageId(2))
///     .content(None)?
///     .await?;
/// # Ok(()) }
/// ```
pub struct UpdateMessage<'a> {
    channel_id: ChannelId,
    fields: UpdateMessageFields,
    fut: Option<Pending<'a, Message>>,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> UpdateMessage<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId, message_id: MessageId) -> Self {
        Self {
            channel_id,
            fields: UpdateMessageFields::default(),
            fut: None,
            http,
            message_id,
        }
    }

    /// Specify an attachment already present in the target message to keep.
    ///
    /// If called, all unspecified attachments will be removed from the message.
    /// If not called, all attachments will be kept.
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.fields.attachments.push(attachment);

        self
    }

    /// Specify multiple attachments already present in the target message to keep.
    ///
    /// If called, all unspecified attachments will be removed from the message.
    /// If not called, all attachments will be kept.
    pub fn attachments(mut self, attachments: impl IntoIterator<Item = Attachment>) -> Self {
        self.fields
            .attachments
            .extend(attachments.into_iter().collect::<Vec<Attachment>>());

        self
    }

    /// Add multiple [`Component`]s to a message.
    ///
    /// Calling this method multiple times will clear previous calls.
    ///
    /// Pass `None` to clear existing components.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateMessageErrorType::ComponentCount`] error type if
    /// too many components are provided.
    ///
    /// Returns an [`UpdateMessageErrorType::ComponentInvalid`] error type if
    /// one of the provided components is invalid.
    pub fn components(
        mut self,
        components: Option<Vec<Component>>,
    ) -> Result<Self, UpdateMessageError> {
        if let Some(components) = components.as_ref() {
            validate_inner::components(&components).map_err(|source| {
                let (kind, inner_source) = source.into_parts();

                match kind {
                    ComponentValidationErrorType::ComponentCount { count } => UpdateMessageError {
                        kind: UpdateMessageErrorType::ComponentCount { count },
                        source: inner_source,
                    },
                    other => UpdateMessageError {
                        kind: UpdateMessageErrorType::ComponentInvalid { kind: other },
                        source: inner_source,
                    },
                }
            })?;
        }

        self.fields
            .components
            .replace(NullableField::from_option(components));

        Ok(self)
    }

    /// Set the content of the message.
    ///
    /// Pass `None` if you want to remove the message content.
    ///
    /// Note that if there is no embed then you will not be able
    /// to remove the content of the message.
    ///
    /// The maximum length is 2000 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateMessageErrorType::ContentInvalid`] error type if the
    /// content length is too long.
    pub fn content(self, content: impl Into<Option<String>>) -> Result<Self, UpdateMessageError> {
        self._content(content.into())
    }

    fn _content(mut self, content: Option<String>) -> Result<Self, UpdateMessageError> {
        if let Some(content_ref) = content.as_ref() {
            if !validate_inner::content_limit(content_ref) {
                return Err(UpdateMessageError {
                    kind: UpdateMessageErrorType::ContentInvalid {
                        content: content.expect("content is known to be some"),
                    },
                    source: None,
                });
            }
        }

        self.fields
            .content
            .replace(NullableField::from_option(content));

        Ok(self)
    }

    /// Attach an embed to the message.
    ///
    /// Pass `None` if you want to remove all of the embeds.
    ///
    /// The first call of this method will clear all present embeds from a
    /// message and replace it with the set embed. Subsequent calls will add
    /// more embeds.
    ///
    /// To pass multiple embeds at once, use [`embeds`].
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateMessageErrorType::EmbedTooLarge`] error type if the
    /// embed is too large.
    ///
    /// [`embeds`]: Self::embeds
    pub fn embed(self, embed: impl Into<Option<Embed>>) -> Result<Self, UpdateMessageError> {
        self._embed(embed.into())
    }

    fn _embed(mut self, embed: Option<Embed>) -> Result<Self, UpdateMessageError> {
        if let Some(embed_ref) = embed.as_ref() {
            validate_inner::embed(&embed_ref)
                .map_err(|source| UpdateMessageError::embed(source, embed_ref.clone(), None))?;
        }

        if let Some(embed) = embed {
            if let Some(embeds) = &mut self.fields.embeds {
                embeds.push(embed);
            } else {
                self.fields.embeds.replace(Vec::from([embed]));
            }
        } else {
            self.fields.embeds.replace(Vec::new());
        }

        Ok(self)
    }

    /// Attach multiple embeds to the message.
    ///
    /// To keep all embeds, do not use this.
    ///
    /// To modify one or more embeds in the message, acquire them from the
    /// previous message, mutate them in place, then pass that list to this
    /// method.
    ///
    /// To remove all embeds, pass an empty iterator via a function like
    /// [`std::iter::empty`].
    ///
    /// Note that if there is no content or file then you will not be able to
    /// remove all of the embeds.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateMessageErrorType::EmbedTooLarge`] error type if an
    /// embed is too large.
    pub fn embeds(
        mut self,
        embeds: impl IntoIterator<Item = Embed>,
    ) -> Result<Self, UpdateMessageError> {
        for (idx, embed) in embeds.into_iter().enumerate() {
            validate_inner::embed(&embed)
                .map_err(|source| UpdateMessageError::embed(source, embed.clone(), Some(idx)))?;

            if let Some(embeds) = &mut self.fields.embeds {
                embeds.push(embed);
            } else {
                self.fields.embeds.replace(Vec::from([embed]));
            }
        }

        Ok(self)
    }

    /// Suppress the embeds in the message.
    pub fn suppress_embeds(mut self, suppress: bool) -> Self {
        let mut flags = self.fields.flags.unwrap_or_else(MessageFlags::empty);

        if suppress {
            flags |= MessageFlags::SUPPRESS_EMBEDS;
        } else {
            flags &= !MessageFlags::SUPPRESS_EMBEDS;
        }
        self.fields.flags.replace(flags);

        self
    }

    /// Set the [`AllowedMentions`] in the message.
    pub fn allowed_mentions(mut self, allowed: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::UpdateMessage {
            channel_id: self.channel_id.0,
            message_id: self.message_id.0,
        })
        .json(&self.fields)?
        .build();

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(UpdateMessage<'_>, Message);
