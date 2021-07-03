use crate::{
    client::Client,
    request::{
        self,
        validate::{self, EmbedValidationError},
        NullableField, Request,
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

    fn embed(source: EmbedValidationError, idx: usize) -> Self {
        Self {
            kind: UpdateMessageErrorType::EmbedTooLarge { idx },
            source: Some(Box::new(source)),
        }
    }
}

impl Display for UpdateMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateMessageErrorType::ContentInvalid { .. } => {
                f.write_str("the message content is invalid")
            }
            UpdateMessageErrorType::EmbedTooLarge { idx, .. } => {
                f.write_str("the embed at index ")?;
                Display::fmt(idx, f)?;

                f.write_str("'s contents are too long")
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
    /// Returned when the content is over 2000 UTF-16 characters.
    ContentInvalid,
    /// Returned when the length of the embed is over 6000 characters.
    EmbedTooLarge {
        /// Index of the embed.
        idx: usize,
    },
}

#[derive(Serialize)]
struct UpdateMessageFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "request::slice_is_empty")]
    pub attachments: &'a [Attachment],
    // We don't serialize if this is Option::None, to avoid overwriting the
    // field without meaning to.
    //
    // So we use a nested Option, representing the following states:
    //
    // - Some(Some(str)): Modifying the "content" from one state to a string;
    // - Some(None): Removing the "content" by giving the Discord API a written
    //   `"content": null` in the JSON;
    // - None: Don't serialize the field at all, not modifying the state.
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<NullableField<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<&'a [Embed]>,
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
/// let client = Client::new("my token".to_owned());
/// client.update_message(ChannelId(1), MessageId(2))
///     .content(Some("test update"))?
///     .exec()
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
/// # let client = Client::new("my token".to_owned());
/// client.update_message(ChannelId(1), MessageId(2))
///     .content(None)?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct UpdateMessage<'a> {
    channel_id: ChannelId,
    fields: UpdateMessageFields<'a>,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> UpdateMessage<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Self {
        Self {
            channel_id,
            fields: UpdateMessageFields {
                allowed_mentions: None,
                attachments: &[],
                content: None,
                embeds: None,
                flags: None,
            },
            http,
            message_id,
        }
    }

    /// Specify multiple attachments already present in the target message to
    /// keep.
    ///
    /// If called, all unspecified attachments will be removed from the message.
    /// If not called, all attachments will be kept.
    ///
    /// Calling this method will clear previous calls.
    pub const fn attachments(mut self, attachments: &'a [Attachment]) -> Self {
        self.fields.attachments = attachments;

        self
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
    pub fn content(mut self, content: Option<&'a str>) -> Result<Self, UpdateMessageError> {
        if let Some(content_ref) = content.as_ref() {
            if !validate::content_limit(content_ref) {
                return Err(UpdateMessageError {
                    kind: UpdateMessageErrorType::ContentInvalid,
                    source: None,
                });
            }
        }

        self.fields.content = Some(NullableField(content));

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
    /// Calling this method again will clear previous calls.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateMessageErrorType::EmbedTooLarge`] error type if an
    /// embed is too large.
    pub fn embeds(mut self, embeds: &'a [Embed]) -> Result<Self, UpdateMessageError> {
        for (idx, embed) in embeds.iter().enumerate() {
            validate::embed(embed).map_err(|source| UpdateMessageError::embed(source, idx))?;
        }

        self.fields.embeds = Some(embeds);

        Ok(self)
    }

    /// Suppress the embeds in the message.
    pub const fn suppress_embeds(mut self, suppress: bool) -> Self {
        #[allow(clippy::option_if_let_else)]
        let mut bits = if let Some(flags) = self.fields.flags {
            flags.bits()
        } else {
            0
        };

        if suppress {
            bits |= MessageFlags::SUPPRESS_EMBEDS.bits();
        } else {
            bits &= !MessageFlags::SUPPRESS_EMBEDS.bits()
        }

        self.fields.flags = Some(MessageFlags::from_bits_truncate(bits));

        self
    }

    /// Set the [`AllowedMentions`] in the message.
    pub fn allowed_mentions(mut self, allowed: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Message> {
        let mut request = Request::builder(Route::UpdateMessage {
            channel_id: self.channel_id.0,
            message_id: self.message_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
