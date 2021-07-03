//! Update a message created by a webhook via execution.

use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, AuditLogReason, AuditLogReasonError, Form, NullableField, Request},
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
    id::{MessageId, WebhookId},
};

/// A webhook's message can not be updated as configured.
#[derive(Debug)]
pub struct UpdateWebhookMessageError {
    kind: UpdateWebhookMessageErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl UpdateWebhookMessageError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateWebhookMessageErrorType {
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
        UpdateWebhookMessageErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, self.source)
    }
}

impl Display for UpdateWebhookMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateWebhookMessageErrorType::ContentInvalid { .. } => {
                f.write_str("message content is invalid")
            }
            UpdateWebhookMessageErrorType::EmbedTooLarge { .. } => {
                f.write_str("length of one of the embeds is too large")
            }
            UpdateWebhookMessageErrorType::TooManyEmbeds { embeds } => {
                Display::fmt(&embeds.len(), f)?;

                f.write_str(" embeds were provided, but only 10 may be provided")
            }
        }
    }
}

impl Error for UpdateWebhookMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`UpdateWebhookMessageError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateWebhookMessageErrorType {
    /// Content is over 2000 UTF-16 characters.
    ContentInvalid {
        /// Provided content.
        content: String,
    },
    /// Length of one of the embeds is over 6000 characters.
    EmbedTooLarge {
        /// Provided embeds.
        embeds: Vec<Embed>,
        /// Index of the embed that was too large.
        ///
        /// This can be used to index into [`embeds`] to retrieve the bad embed.
        ///
        /// [`embeds`]: Self::EmbedTooLarge.embeds
        index: usize,
    },
    /// Too many embeds were provided.
    ///
    /// A webhook can have up to 10 embeds.
    TooManyEmbeds {
        /// Provided embeds.
        embeds: Vec<Embed>,
    },
}

#[derive(Default, Serialize)]
struct UpdateWebhookMessageFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<NullableField<Vec<Embed>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<Vec<u8>>,
}

/// Update a message created by a webhook.
///
/// A webhook's message must always have at least one embed or some amount of
/// content. If you wish to delete a webhook's message refer to
/// [`DeleteWebhookMessage`].
///
/// # Examples
///
/// Update a webhook's message by setting the content to `test <@3>` -
/// attempting to mention user ID 3 - and specifying that only that the user may
/// not be mentioned.
///
/// ```no_run
/// # use twilight_http::Client;
/// use twilight_model::{
///     channel::message::AllowedMentions,
///     id::{MessageId, WebhookId}
/// };
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = Client::new("token");
/// client.update_webhook_message(WebhookId(1), "token here", MessageId(2))
///     // By creating a default set of allowed mentions, no entity can be
///     // mentioned.
///     .allowed_mentions(AllowedMentions::default())
///     .content(Some("test <@3>".to_owned()))?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`DeleteWebhookMessage`]: super::DeleteWebhookMessage
pub struct UpdateWebhookMessage<'a> {
    fields: UpdateWebhookMessageFields,
    files: Vec<(String, Vec<u8>)>,
    http: &'a Client,
    message_id: MessageId,
    reason: Option<String>,
    token: String,
    webhook_id: WebhookId,
}

impl<'a> UpdateWebhookMessage<'a> {
    /// Maximum number of embeds that a webhook's message may have.
    pub const EMBED_COUNT_LIMIT: usize = 10;

    pub(crate) fn new(
        http: &'a Client,
        webhook_id: WebhookId,
        token: impl Into<String>,
        message_id: MessageId,
    ) -> Self {
        Self {
            fields: UpdateWebhookMessageFields {
                allowed_mentions: http.default_allowed_mentions(),
                ..UpdateWebhookMessageFields::default()
            },
            files: Vec::new(),
            http,
            message_id,
            reason: None,
            token: token.into(),
            webhook_id,
        }
    }

    /// Set the allowed mentions in the message.
    pub fn allowed_mentions(mut self, allowed: AllowedMentions) -> Self {
        self.fields.allowed_mentions.replace(allowed);

        self
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
    /// Returns an [`UpdateWebhookMessageErrorType::ContentInvalid`] error type if
    /// the content length is too long.
    pub fn content(mut self, content: Option<String>) -> Result<Self, UpdateWebhookMessageError> {
        if let Some(content_ref) = content.as_ref() {
            if !validate::content_limit(content_ref) {
                return Err(UpdateWebhookMessageError {
                    kind: UpdateWebhookMessageErrorType::ContentInvalid {
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

    /// Set the list of embeds of the webhook's message.
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
    /// # use twilight_http::Client;
    /// use twilight_embed_builder::EmbedBuilder;
    /// use twilight_model::id::{MessageId, WebhookId};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// let embed = EmbedBuilder::new()
    ///     .description("Powerful, flexible, and scalable ecosystem of Rust libraries for the Discord API.")
    ///     .title("Twilight")
    ///     .url("https://twilight.rs")
    ///     .build()?;
    ///
    /// client.update_webhook_message(WebhookId(1), "token", MessageId(2))
    ///     .embeds(Some(vec![embed]))?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateWebhookMessageErrorType::EmbedTooLarge`] error type
    /// if one of the embeds are too large.
    ///
    /// Returns an [`UpdateWebhookMessageErrorType::TooManyEmbeds`] error type
    /// if more than 10 embeds are provided.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#embed-limits
    /// [`EMBED_COUNT_LIMIT`]: Self::EMBED_COUNT_LIMIT
    pub fn embeds(mut self, embeds: Option<Vec<Embed>>) -> Result<Self, UpdateWebhookMessageError> {
        if let Some(embeds_present) = embeds.as_deref() {
            if embeds_present.len() > Self::EMBED_COUNT_LIMIT {
                return Err(UpdateWebhookMessageError {
                    kind: UpdateWebhookMessageErrorType::TooManyEmbeds {
                        embeds: embeds.expect("embeds are known to be present"),
                    },
                    source: None,
                });
            }

            for (idx, embed) in embeds_present.iter().enumerate() {
                if let Err(source) = validate::embed(&embed) {
                    return Err(UpdateWebhookMessageError {
                        kind: UpdateWebhookMessageErrorType::EmbedTooLarge {
                            embeds: embeds.expect("embeds are known to be present"),
                            index: idx,
                        },
                        source: Some(Box::new(source)),
                    });
                }
            }
        }

        self.fields
            .embeds
            .replace(NullableField::from_option(embeds));

        Ok(self)
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
    /// [`file`]. See [Discord Docs/Create Message] and
    /// [`ExecuteWebhook::payload_json`].
    ///
    /// [`file`]: Self::file
    /// [`ExecuteWebhook::payload_json`]: super::ExecuteWebhook::payload_json
    /// [Discord Docs/Create Message]: https://discord.com/developers/docs/resources/channel#create-message-params
    pub fn payload_json(mut self, payload_json: impl Into<Vec<u8>>) -> Self {
        self.fields.payload_json.replace(payload_json.into());

        self
    }

    fn request(self) -> Result<(Request, &'a Client), HttpError> {
        let mut request = Request::builder(Route::UpdateWebhookMessage {
            message_id: self.message_id.0,
            token: self.token,
            webhook_id: self.webhook_id.0,
        })
        .use_authorization_token(false);

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.iter().enumerate() {
                form.file(format!("{}", index).as_bytes(), name.as_bytes(), file);
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

        if let Some(reason) = self.reason.as_ref() {
            request = request.headers(request::audit_header(reason)?);
        }

        Ok((request.build(), self.http))
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        match self.request() {
            Ok((request, client)) => client.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason for UpdateWebhookMessage<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{UpdateWebhookMessage, UpdateWebhookMessageFields};
    use crate::{
        client::Client,
        request::{AuditLogReason, NullableField, Request},
        routing::Route,
    };
    use twilight_model::id::{MessageId, WebhookId};

    #[test]
    fn test_request() {
        let client = Client::new("token");
        let builder = UpdateWebhookMessage::new(&client, WebhookId(1), "token", MessageId(2))
            .content(Some("test".to_owned()))
            .expect("'test' content couldn't be set")
            .reason("reason")
            .expect("'reason' is not a valid reason");
        let (actual, _) = builder.request().expect("failed to create request");

        let body = UpdateWebhookMessageFields {
            allowed_mentions: None,
            attachments: Vec::new(),
            content: Some(NullableField::Value("test".to_owned())),
            embeds: None,
            payload_json: None,
        };
        let route = Route::UpdateWebhookMessage {
            message_id: 2,
            token: "token".to_owned(),
            webhook_id: 1,
        };
        let expected = Request::builder(route)
            .json(&body)
            .expect("failed to serialize body")
            .build();

        assert_eq!(expected.body, actual.body);
        assert_eq!(expected.path, actual.path);
    }
}
