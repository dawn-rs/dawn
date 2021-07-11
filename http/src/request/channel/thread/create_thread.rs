use super::{ThreadValidationError, ThreadValidationErrorType};
use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate, Pending, Request},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{thread::AutoArchiveDuration, Channel, ChannelType},
    id::ChannelId,
};

#[derive(Serialize)]
struct CreateThreadFields {
    auto_archive_duration: AutoArchiveDuration,
    #[serde(rename = "type")]
    kind: ChannelType,
    name: String,
}

/// Start a thread that is not connected to a message.
///
/// To use auto archive durations of [`ThreeDays`] or [`Week`], the guild must
/// be boosted.
///
/// [`ThreeDays`]: twilight_model::channel::thread::AutoArchiveDuration::ThreeDays
/// [`Week`]: twilight_model::channel::thread::AutoArchiveDuration::Week
pub struct CreateThread<'a> {
    channel_id: ChannelId,
    fields: CreateThreadFields,
    fut: Option<Pending<'a, Channel>>,
    http: &'a Client,
}

impl<'a> CreateThread<'a> {
    pub(crate) fn new(
        http: &'a Client,
        channel_id: ChannelId,
        name: impl Into<String>,
        auto_archive_duration: AutoArchiveDuration,
        kind: ChannelType,
    ) -> Result<Self, ThreadValidationError> {
        Self::_new(http, channel_id, name.into(), auto_archive_duration, kind)
    }

    fn _new(
        http: &'a Client,
        channel_id: ChannelId,
        name: String,
        auto_archive_duration: AutoArchiveDuration,
        kind: ChannelType,
    ) -> Result<Self, ThreadValidationError> {
        if !validate::channel_name(&name) {
            return Err(ThreadValidationError {
                kind: ThreadValidationErrorType::NameInvalid { name },
            });
        }

        if !validate::is_thread(kind) {
            return Err(ThreadValidationError {
                kind: ThreadValidationErrorType::TypeInvalid { kind },
            });
        }

        Ok(Self {
            channel_id,
            fields: CreateThreadFields {
                auto_archive_duration,
                kind,
                name,
            },
            fut: None,
            http,
        })
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::CreateThread {
            channel_id: self.channel_id.0,
        })
        .json(&self.fields)?;

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

poll_req!(CreateThread<'_>, Channel);
