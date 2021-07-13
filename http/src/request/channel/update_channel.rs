use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        self, validate_inner, AuditLogReason, AuditLogReasonError, NullableField, Pending, Request,
    },
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, Channel, ChannelType, VideoQualityMode},
    id::ChannelId,
};

/// Returned when the channel can not be updated as configured.
#[derive(Debug)]
pub struct UpdateChannelError {
    kind: UpdateChannelErrorType,
}

impl UpdateChannelError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateChannelErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (UpdateChannelErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for UpdateChannelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateChannelErrorType::NameInvalid { .. } => {
                f.write_str("the length of the name is invalid")
            }
            UpdateChannelErrorType::RateLimitPerUserInvalid { .. } => {
                f.write_str("the rate limit per user is invalid")
            }
            UpdateChannelErrorType::TopicInvalid { .. } => f.write_str("the topic is invalid"),
        }
    }
}

impl Error for UpdateChannelError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateChannelErrorType {
    /// The length of the name is either fewer than 1 UTF-16 character or more
    /// than 100 UTF-16 characters.
    NameInvalid {
        /// Provided name.
        name: String,
    },
    /// The seconds of the rate limit per user is more than 21600.
    RateLimitPerUserInvalid {
        /// Provided ratelimit is invalid.
        rate_limit_per_user: u64,
    },
    /// The length of the topic is more than 1024 UTF-16 characters.
    TopicInvalid {
        /// Provided topic.
        topic: String,
    },
}

// The Discord API doesn't require the `name` and `kind` fields to be present,
// but it does require them to be non-null.
#[derive(Default, Serialize)]
struct UpdateChannelFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    bitrate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permission_overwrites: Option<Vec<PermissionOverwrite>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_quality_mode: Option<VideoQualityMode>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<ChannelType>,
}

/// Update a channel.
///
/// All fields are optional. The minimum length of the name is 1 UTF-16 character
/// and the maximum is 100 UTF-16 characters.
pub struct UpdateChannel<'a> {
    channel_id: ChannelId,
    fields: UpdateChannelFields,
    fut: Option<Pending<'a, Channel>>,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> UpdateChannel<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: UpdateChannelFields::default(),
            fut: None,
            http,
            reason: None,
        }
    }

    /// Set the bitrate of the channel. Applicable to voice channels only.
    pub fn bitrate(mut self, bitrate: u64) -> Self {
        self.fields.bitrate.replace(bitrate);

        self
    }

    /// Set the name.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 100 UTF-16
    /// characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateChannelErrorType::NameInvalid`] error type if the name
    /// length is too short or too long.
    pub fn name(self, name: impl Into<String>) -> Result<Self, UpdateChannelError> {
        self._name(name.into())
    }

    fn _name(mut self, name: String) -> Result<Self, UpdateChannelError> {
        if !validate_inner::channel_name(&name) {
            return Err(UpdateChannelError {
                kind: UpdateChannelErrorType::NameInvalid { name },
            });
        }

        self.fields.name.replace(name);

        Ok(self)
    }

    /// Set whether the channel is marked as NSFW.
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.fields.nsfw.replace(nsfw);

        self
    }

    /// If this is specified, and the parent ID is a `ChannelType::CategoryChannel`, move this
    /// channel to a child of the category channel.
    pub fn parent_id(mut self, parent_id: impl Into<Option<ChannelId>>) -> Self {
        let parent_id = parent_id.into();

        self.fields
            .parent_id
            .replace(NullableField::from_option(parent_id));

        self
    }

    /// Set the permission overwrites of a channel. This will overwrite all permissions that the
    /// channel currently has, so use with caution!
    pub fn permission_overwrites(
        mut self,
        permission_overwrites: Vec<PermissionOverwrite>,
    ) -> Self {
        self.fields
            .permission_overwrites
            .replace(permission_overwrites);

        self
    }

    /// Set the position of the channel.
    ///
    /// Positions are numerical and zero-indexed. If you place a channel at position 2, channels
    /// 2-n will shift down one position and the initial channel will take its place.
    pub fn position(mut self, position: u64) -> Self {
        self.fields.position.replace(position);

        self
    }

    /// Set the number of seconds that a user must wait before before they are able to send another
    /// message.
    ///
    /// The minimum is 0 and the maximum is 21600. Refer to [the discord docs] for more details.
    /// This is also known as "Slow Mode".
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateChannelErrorType::RateLimitPerUserInvalid`] error
    /// type if the amount is greater than 21600.
    ///
    /// [the discord docs]: https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure>
    pub fn rate_limit_per_user(
        mut self,
        rate_limit_per_user: u64,
    ) -> Result<Self, UpdateChannelError> {
        if rate_limit_per_user > 21600 {
            return Err(UpdateChannelError {
                kind: UpdateChannelErrorType::RateLimitPerUserInvalid {
                    rate_limit_per_user,
                },
            });
        }

        self.fields.rate_limit_per_user.replace(rate_limit_per_user);

        Ok(self)
    }

    /// Set the topic.
    ///
    /// The maximum length is 1024 UTF-16 characters. Refer to [the discord docs] for more details.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateChannelErrorType::TopicInvalid`] error type if the topic
    /// length is too long.
    ///
    /// [the discord docs]: https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure
    pub fn topic(self, topic: impl Into<String>) -> Result<Self, UpdateChannelError> {
        self._topic(topic.into())
    }

    fn _topic(mut self, topic: String) -> Result<Self, UpdateChannelError> {
        if topic.chars().count() > 1024 {
            return Err(UpdateChannelError {
                kind: UpdateChannelErrorType::TopicInvalid { topic },
            });
        }

        self.fields.topic.replace(topic);

        Ok(self)
    }

    /// For voice channels, set the user limit.
    ///
    /// Set to 0 for no limit. Limit can otherwise be between 1 and 99 inclusive. Refer to [the
    /// discord docs] for more details.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#modify-channel-json-params
    pub fn user_limit(mut self, user_limit: u64) -> Self {
        self.fields.user_limit.replace(user_limit);

        self
    }

    /// Set the [`VideoQualityMode`] for the voice channel.
    pub fn video_quality_mode(mut self, video_quality_mode: VideoQualityMode) -> Self {
        self.fields.video_quality_mode.replace(video_quality_mode);

        self
    }

    /// Set the kind of channel.
    ///
    /// Only conversion between `ChannelType::GuildText` and `ChannelType::GuildNews` is possible,
    /// and only if the guild has the `NEWS` feature enabled. Refer to [the discord docs] for more
    /// details.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#modify-channel-json-params
    pub fn kind(mut self, kind: ChannelType) -> Self {
        self.fields.kind.replace(kind);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let mut request = Request::builder(Route::UpdateChannel {
            channel_id: self.channel_id.0,
        });

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for UpdateChannel<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(UpdateChannel<'_>, Channel);
