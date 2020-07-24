use crate::request::prelude::*;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::{
        DefaultMessageNotificationLevel, ExplicitContentFilter, PartialGuild, VerificationLevel,
    },
    id::{ChannelId, GuildId, UserId},
};

/// The error returned when the guild can not be updated as configured.
#[derive(Clone, Debug)]
pub enum UpdateGuildError {
    /// The name length is either fewer than 2 UTF-16 characters or more than 100 UTF-16
    /// characters.
    NameInvalid,
}

impl Display for UpdateGuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NameInvalid => f.write_str("the name's length is invalid"),
        }
    }
}

impl Error for UpdateGuildError {}

#[derive(Default, Serialize)]
struct UpdateGuildFields {
    afk_channel_id: Option<ChannelId>,
    afk_timeout: Option<u64>,
    default_message_notifications: Option<DefaultMessageNotificationLevel>,
    explicit_content_filter: Option<ExplicitContentFilter>,
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<UserId>,
    region: Option<String>,
    splash: Option<String>,
    system_channel_id: Option<ChannelId>,
    verification_level: Option<VerificationLevel>,
    rules_channel_id: Option<ChannelId>,
    public_updates_channel_id: Option<ChannelId>,
    preferred_locale: Option<String>,
}

/// Update a guild.
///
/// All endpoints are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild
pub struct UpdateGuild<'a> {
    fields: UpdateGuildFields,
    fut: Option<Pending<'a, PartialGuild>>,
    guild_id: GuildId,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> UpdateGuild<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildFields::default(),
            fut: None,
            guild_id,
            http,
            reason: None,
        }
    }

    /// Set the voice channel where AFK voice users are sent.
    pub fn afk_channel_id(mut self, afk_channel_id: impl Into<ChannelId>) -> Self {
        self.fields.afk_channel_id.replace(afk_channel_id.into());

        self
    }

    /// Set how much time it takes for a voice user to be considered AFK.
    pub fn afk_timeout(mut self, afk_timeout: u64) -> Self {
        self.fields.afk_timeout.replace(afk_timeout);

        self
    }

    /// Set the default message notification level. Refer to [the discord docs] for more
    /// information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#create-guild
    pub fn default_message_notifications(
        mut self,
        default_message_notifications: DefaultMessageNotificationLevel,
    ) -> Self {
        self.fields
            .default_message_notifications
            .replace(default_message_notifications);

        self
    }

    /// Set the explicit content filter level.
    pub fn explicit_content_filter(
        mut self,
        explicit_content_filter: ExplicitContentFilter,
    ) -> Self {
        self.fields
            .explicit_content_filter
            .replace(explicit_content_filter);

        self
    }

    /// Set the icon.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.fields.icon.replace(icon.into());

        self
    }

    /// Set the name of the guild.
    ///
    /// The minimum length is 2 UTF-16 characters and the maximum is 100 UTF-16
    /// characters.
    ///
    /// # Erroors
    ///
    /// Returns [`UpdateGuildError::NameInvalid`] if the name length is too
    /// short or too long.
    ///
    /// [`UpdateGuildError::NameInvalid`]: enum.UpdateGuildError.html#variant.NameInvalid
    pub fn name(self, name: impl Into<String>) -> Result<Self, UpdateGuildError> {
        self._name(name.into())
    }

    fn _name(mut self, name: String) -> Result<Self, UpdateGuildError> {
        if !validate::guild_name(&name) {
            return Err(UpdateGuildError::NameInvalid);
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

    /// Specify the voice server region for the guild. Refer to [the discord docs] for more
    /// information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/voice#voice-region-object
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.fields.region.replace(region.into());

        self
    }

    /// Set the guild's splash image.
    ///
    /// Requires the guild to have the `INVITE_SPLASH` feature enabled.
    pub fn splash(mut self, splash: impl Into<String>) -> Self {
        self.fields.splash.replace(splash.into());

        self
    }

    /// Set the channel where events such as welcome messages are posted.
    pub fn system_channel(mut self, system_channel_id: impl Into<ChannelId>) -> Self {
        self.fields
            .system_channel_id
            .replace(system_channel_id.into());

        self
    }

    /// Set the rules channel.
    ///
    /// Requires the guild to be `PUBLIC`. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild
    pub fn rules_channel(mut self, rules_channel_id: impl Into<ChannelId>) -> Self {
        self.fields
            .rules_channel_id
            .replace(rules_channel_id.into());

        self
    }

    /// Set the public updates channel.
    ///
    /// Requires the guild to be `PUBLIC`.
    pub fn public_updates_channel(
        mut self,
        public_updates_channel_id: impl Into<ChannelId>,
    ) -> Self {
        self.fields
            .public_updates_channel_id
            .replace(public_updates_channel_id.into());

        self
    }

    /// Set the preferred locale for the guild.
    ///
    /// Defaults to `en-US`. Requires the guild to be `PUBLIC`.
    pub fn preferred_locale(mut self, preferred_locale: impl Into<String>) -> Self {
        self.fields
            .preferred_locale
            .replace(preferred_locale.into());

        self
    }

    /// Set the verification level. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#guild-object-verification-level
    pub fn verification_level(mut self, verification_level: VerificationLevel) -> Self {
        self.fields.verification_level.replace(verification_level);

        self
    }

    /// Attach an audit log reason to this request.
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason.replace(reason.into());

        self
    }

    fn start(&mut self) -> Result<()> {
        let request = if let Some(reason) = &self.reason {
            let headers = audit_header(&reason)?;
            Request::from((
                crate::json_to_vec(&self.fields)?,
                headers,
                Route::UpdateGuild {
                    guild_id: self.guild_id.0,
                },
            ))
        } else {
            Request::from((
                crate::json_to_vec(&self.fields)?,
                Route::UpdateGuild {
                    guild_id: self.guild_id.0,
                },
            ))
        };

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(UpdateGuild<'_>, PartialGuild);
