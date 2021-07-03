use crate::{
    client::Client,
    error::Error,
    request::{Form, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{
        embed::Embed,
        message::{AllowedMentions, MessageFlags},
        Message,
    },
    id::ApplicationId,
};

#[derive(Default, Serialize)]
pub(crate) struct CreateFollowupMessageFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<&'a [Embed]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload_json: Option<&'a [u8]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    allowed_mentions: Option<&'a AllowedMentions>,
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
/// client
///     .create_followup_message("webhook token")?
///     .content("Pinkie...")
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`content`]: Self::content
/// [`embeds`]: Self::embeds
/// [`files`]: Self::files
pub struct CreateFollowupMessage<'a> {
    pub(crate) fields: CreateFollowupMessageFields<'a>,
    files: &'a [(&'a str, &'a [u8])],
    http: &'a Client,
    token: &'a str,
    application_id: ApplicationId,
}

impl<'a> CreateFollowupMessage<'a> {
    pub(crate) fn new(http: &'a Client, application_id: ApplicationId, token: &'a str) -> Self {
        Self {
            fields: CreateFollowupMessageFields::default(),
            files: &[],
            http,
            token,
            application_id,
        }
    }

    /// Specify the [`AllowedMentions`] for the webhook message.
    pub const fn allowed_mentions(mut self, allowed_mentions: &'a AllowedMentions) -> Self {
        self.fields.allowed_mentions = Some(allowed_mentions);

        self
    }

    /// The URL of the avatar of the webhook.
    pub const fn avatar_url(mut self, avatar_url: &'a str) -> Self {
        self.fields.avatar_url = Some(avatar_url);

        self
    }

    /// The content of the webook's message.
    ///
    /// Up to 2000 UTF-16 codepoints.
    pub fn content(mut self, content: &'a str) -> Self {
        self.fields.content.replace(content);

        self
    }

    /// Set the list of embeds of the webhook's message.
    pub const fn embeds(mut self, embeds: &'a [Embed]) -> Self {
        self.fields.embeds = Some(embeds);

        self
    }

    /// Set if the followup should be ephemeral.
    pub const fn ephemeral(mut self, ephemeral: bool) -> Self {
        if ephemeral {
            self.fields.flags = Some(MessageFlags::EPHEMERAL);
        } else {
            self.fields.flags = None;
        }

        self
    }

    /// Attach multiple files to the webhook.
    pub const fn files(mut self, files: &'a [(&'a str, &'a [u8])]) -> Self {
        self.files = files;

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
    ///     .embeds(&[EmbedBuilder::new().title("title").build()?])
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
    ///     .payload_json(br#"{ "content": "other content", "embeds": [ { "title": "title" } ] }"#)
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
    pub const fn payload_json(mut self, payload_json: &'a [u8]) -> Self {
        self.fields.payload_json = Some(payload_json);

        self
    }

    /// Specify true if the message is TTS.
    pub const fn tts(mut self, tts: bool) -> Self {
        self.fields.tts = Some(tts);

        self
    }

    /// Specify the username of the webhook's message.
    pub const fn username(mut self, username: &'a str) -> Self {
        self.fields.username = Some(username);

        self
    }

    fn request(&mut self) -> Result<Request<'a>, Error> {
        let mut request = Request::builder(Route::ExecuteWebhook {
            token: self.token,
            wait: None,
            webhook_id: self.application_id.0,
        });

        if !self.files.is_empty() || self.fields.payload_json.is_some() {
            let mut form = Form::new();

            for (index, (name, file)) in self.files.iter().enumerate() {
                form.file(index.to_be_bytes().as_ref(), name.as_bytes(), file);
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

        Ok(request.build())
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(mut self) -> ResponseFuture<Message> {
        match self.request() {
            Ok(request) => self.http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}
