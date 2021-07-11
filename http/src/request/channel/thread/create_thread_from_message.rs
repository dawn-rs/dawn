use super::{ThreadValidationError, ThreadValidationErrorType};
use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate, Pending, Request},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{thread::AutoArchiveDuration, Channel},
    id::{ChannelId, MessageId},
};

#[derive(Serialize)]
struct CreateThreadFromMessageFields {
    auto_archive_duration: AutoArchiveDuration,
    name: String,
}

/// Create a new thread from an existing message.
///
/// When called on a [`GuildText`] channel, this creates a
/// [`GuildPublicThread`].
///
/// When called on a [`GuildNews`] channel, this creates a [`GuildNewsThread`].
///
/// To use auto archive durations of [`ThreeDays`] or [`Week`], the guild must
/// be boosted.
///
/// The thread's ID will be the same as its parent message. This ensures only
/// one thread can be created per message.
///
/// [`GuildNewsThread`]: twilight_model::channel::ChannelType::GuildNewsThread
/// [`GuildNews`]: twilight_model::channel::ChannelType::GuildNews
/// [`GuildPublicThread`]: twilight_model::channel::ChannelType::GuildPublicThread
/// [`GuildText`]: twilight_model::channel::ChannelType::GuildText
/// [`ThreeDays`]: twilight_model::channel::thread::AutoArchiveDuration::ThreeDays
/// [`Week`]: twilight_model::channel::thread::AutoArchiveDuration::Week
pub struct CreateThreadFromMessage<'a> {
    channel_id: ChannelId,
    fields: CreateThreadFromMessageFields,
    fut: Option<Pending<'a, Channel>>,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> CreateThreadFromMessage<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        name: impl Into<String>,
        auto_archive_duration: AutoArchiveDuration,
    ) -> Result<Self, ThreadValidationError> {
        Self::_new(
            http,
            channel_id,
            message_id,
            name.into(),
            auto_archive_duration,
        )
    }

    fn _new(
        http: &'a Client,
        channel_id: ChannelId,
        message_id: MessageId,
        name: String,
        auto_archive_duration: AutoArchiveDuration,
    ) -> Result<Self, ThreadValidationError> {
        if !validate::channel_name(&name) {
            return Err(ThreadValidationError {
                kind: ThreadValidationErrorType::NameInvalid { name },
            });
        }

        Ok(Self {
            channel_id,
            fields: CreateThreadFromMessageFields {
                auto_archive_duration,
                name,
            },
            fut: None,
            http,
            message_id,
        })
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::CreateThreadFromMessage {
            channel_id: self.channel_id.0,
            message_id: self.message_id.0,
        })
        .json(&self.fields)?;

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

poll_req!(CreateThreadFromMessage<'_>, Channel);
