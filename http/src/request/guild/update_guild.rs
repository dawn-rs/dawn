use crate::{
    client::Client,
    request::{self, validate, AuditLogReason, AuditLogReasonError, NullableField, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::{
        DefaultMessageNotificationLevel, ExplicitContentFilter, PartialGuild, SystemChannelFlags,
        VerificationLevel,
    },
    id::{ChannelId, GuildId, UserId},
};

/// The error returned when the guild can not be updated as configured.
#[derive(Debug)]
pub struct UpdateGuildError {
    kind: UpdateGuildErrorType,
}

impl UpdateGuildError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateGuildErrorType {
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
    pub fn into_parts(self) -> (UpdateGuildErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for UpdateGuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateGuildErrorType::NameInvalid { .. } => f.write_str("the name's length is invalid"),
        }
    }
}

impl Error for UpdateGuildError {}

/// Type of [`UpdateGuildError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateGuildErrorType {
    /// The name length is either fewer than 2 UTF-16 characters or more than 100 UTF-16
    /// characters.
    NameInvalid {
        /// Provided name.
        name: String,
    },
}

#[derive(Default, Serialize)]
struct UpdateGuildFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_message_notifications: Option<NullableField<DefaultMessageNotificationLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovery_splash: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_content_filter: Option<NullableField<ExplicitContentFilter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    splash: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_flags: Option<NullableField<SystemChannelFlags>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_level: Option<NullableField<VerificationLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules_channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_updates_channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred_locale: Option<NullableField<String>>,
}

/// Update a guild.
///
/// All endpoints are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild
pub struct UpdateGuild<'a> {
    fields: UpdateGuildFields,
    guild_id: GuildId,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> UpdateGuild<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildFields::default(),
            guild_id,
            http,
            reason: None,
        }
    }

    /// Set the voice channel where AFK voice users are sent.
    pub fn afk_channel_id(mut self, afk_channel_id: impl Into<Option<ChannelId>>) -> Self {
        self.fields
            .afk_channel_id
            .replace(NullableField::from_option(afk_channel_id.into()));

        self
    }

    /// Set how much time it takes for a voice user to be considered AFK.
    pub fn afk_timeout(mut self, afk_timeout: u64) -> Self {
        self.fields.afk_timeout.replace(afk_timeout);

        self
    }

    /// Set the banner.
    ///
    /// This is a base64 encoded 16:9 PNG or JPEG image. Pass `None` to remove
    /// the banner.
    ///
    /// The server must have the `BANNER` feature.
    pub fn banner(mut self, banner: impl Into<Option<String>>) -> Self {
        self.fields
            .banner
            .replace(NullableField::from_option(banner.into()));

        self
    }

    /// Set the default message notification level. Refer to [the discord docs] for more
    /// information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#create-guild
    pub fn default_message_notifications(
        mut self,
        default_message_notifications: impl Into<Option<DefaultMessageNotificationLevel>>,
    ) -> Self {
        self.fields
            .default_message_notifications
            .replace(NullableField::from_option(
                default_message_notifications.into(),
            ));

        self
    }

    /// Set the guild's discovery splash image.
    ///
    /// Requires the guild to have the `DISCOVERABLE` feature enabled.
    pub fn discovery_splash(mut self, discovery_splash: impl Into<Option<String>>) -> Self {
        self.fields
            .discovery_splash
            .replace(NullableField::from_option(discovery_splash.into()));

        self
    }

    /// Set the explicit content filter level.
    pub fn explicit_content_filter(
        mut self,
        explicit_content_filter: impl Into<Option<ExplicitContentFilter>>,
    ) -> Self {
        self.fields
            .explicit_content_filter
            .replace(NullableField::from_option(explicit_content_filter.into()));

        self
    }

    /// Set the enabled features of the guild.
    pub fn features(mut self, features: impl IntoIterator<Item = String>) -> Self {
        self.fields.features.replace(features.into_iter().collect());

        self
    }

    /// Set the icon.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn icon(mut self, icon: impl Into<Option<String>>) -> Self {
        self.fields
            .icon
            .replace(NullableField::from_option(icon.into()));

        self
    }

    /// Set the name of the guild.
    ///
    /// The minimum length is 2 UTF-16 characters and the maximum is 100 UTF-16
    /// characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateGuildErrorType::NameInvalid`] error type if the name
    /// length is too short or too long.
    pub fn name(self, name: impl Into<String>) -> Result<Self, UpdateGuildError> {
        self._name(name.into())
    }

    fn _name(mut self, name: String) -> Result<Self, UpdateGuildError> {
        if !validate::guild_name(&name) {
            return Err(UpdateGuildError {
                kind: UpdateGuildErrorType::NameInvalid { name },
            });
        }

        self.fields.name.replace(name);

        Ok(self)
    }

    /// Transfer ownership to another user.
    ///
    /// Only works if the current user is the owner.
    pub fn owner_id(mut self, owner_id: impl Into<UserId>) -> Self {
        self.fields.owner_id.replace(owner_id.into());

        self
    }

    /// Set the guild's splash image.
    ///
    /// Requires the guild to have the `INVITE_SPLASH` feature enabled.
    pub fn splash(mut self, splash: impl Into<Option<String>>) -> Self {
        self.fields
            .splash
            .replace(NullableField::from_option(splash.into()));

        self
    }

    /// Set the channel where events such as welcome messages are posted.
    pub fn system_channel(mut self, system_channel_id: impl Into<Option<ChannelId>>) -> Self {
        self.fields
            .system_channel_id
            .replace(NullableField::from_option(system_channel_id.into()));

        self
    }

    /// Set the guild's [`SystemChannelFlags`].
    pub fn system_channel_flags(
        mut self,
        system_channel_flags: impl Into<Option<SystemChannelFlags>>,
    ) -> Self {
        self.fields
            .system_channel_flags
            .replace(NullableField::from_option(system_channel_flags.into()));

        self
    }

    /// Set the rules channel.
    ///
    /// Requires the guild to be `PUBLIC`. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild
    pub fn rules_channel(mut self, rules_channel_id: impl Into<Option<ChannelId>>) -> Self {
        self.fields
            .rules_channel_id
            .replace(NullableField::from_option(rules_channel_id.into()));

        self
    }

    /// Set the public updates channel.
    ///
    /// Requires the guild to be `PUBLIC`.
    pub fn public_updates_channel(
        mut self,
        public_updates_channel_id: impl Into<Option<ChannelId>>,
    ) -> Self {
        self.fields
            .public_updates_channel_id
            .replace(NullableField::from_option(public_updates_channel_id.into()));

        self
    }

    /// Set the preferred locale for the guild.
    ///
    /// Defaults to `en-US`. Requires the guild to be `PUBLIC`.
    pub fn preferred_locale(mut self, preferred_locale: impl Into<Option<String>>) -> Self {
        self.fields
            .preferred_locale
            .replace(NullableField::from_option(preferred_locale.into()));

        self
    }

    /// Set the verification level. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#guild-object-verification-level
    pub fn verification_level(
        mut self,
        verification_level: impl Into<Option<VerificationLevel>>,
    ) -> Self {
        self.fields
            .verification_level
            .replace(NullableField::from_option(verification_level.into()));

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<PartialGuild> {
        let mut request = Request::builder(Route::UpdateGuild {
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        if let Some(reason) = &self.reason {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason for UpdateGuild<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
