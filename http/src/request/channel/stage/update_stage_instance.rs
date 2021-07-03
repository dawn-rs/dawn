use crate::{
    client::Client,
    request::{validate, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{channel::stage_instance::PrivacyLevel, id::ChannelId};

/// The request can not be created as configured.
#[derive(Debug)]
pub struct UpdateStageInstanceError {
    kind: UpdateStageInstanceErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl UpdateStageInstanceError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateStageInstanceErrorType {
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
        UpdateStageInstanceErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for UpdateStageInstanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateStageInstanceErrorType::InvalidTopic { .. } => f.write_str("invalid topic"),
        }
    }
}

impl Error for UpdateStageInstanceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

#[derive(Debug)]
pub enum UpdateStageInstanceErrorType {
    /// Topic is not between 1 and 120 characters in length.
    InvalidTopic {
        /// Invalid topic.
        topic: String,
    },
}

#[derive(Default, Serialize)]
struct UpdateStageInstanceFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    privacy_level: Option<PrivacyLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
}

/// Update fields of an existing stage instance.
///
/// Requires the user to be a moderator of the stage channel.
pub struct UpdateStageInstance<'a> {
    channel_id: ChannelId,
    fields: UpdateStageInstanceFields,
    http: &'a Client,
}

impl<'a> UpdateStageInstance<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: UpdateStageInstanceFields::default(),
            http,
        }
    }

    /// Set the [`PrivacyLevel`] of the instance.
    pub fn privacy_level(mut self, privacy_level: PrivacyLevel) -> Self {
        self.fields.privacy_level.replace(privacy_level);

        self
    }

    /// Set the new topic of the instance.
    pub fn topic(self, topic: impl Into<String>) -> Result<Self, UpdateStageInstanceError> {
        self._topic(topic.into())
    }

    fn _topic(mut self, topic: String) -> Result<Self, UpdateStageInstanceError> {
        if !validate::stage_topic(&topic) {
            return Err(UpdateStageInstanceError {
                kind: UpdateStageInstanceErrorType::InvalidTopic { topic },
                source: None,
            });
        }

        self.fields.topic.replace(topic);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::UpdateStageInstance {
            channel_id: self.channel_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
