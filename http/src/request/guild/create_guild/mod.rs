use crate::{
    client::Client,
    request::{validate, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    guild::{
        DefaultMessageNotificationLevel, ExplicitContentFilter, PartialGuild, Permissions,
        SystemChannelFlags, VerificationLevel,
    },
    id::{ChannelId, RoleId},
};

mod builder;

pub use self::builder::*;

/// The error returned when the guild can not be created as configured.
#[derive(Debug)]
pub struct CreateGuildError {
    kind: CreateGuildErrorType,
}

impl CreateGuildError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateGuildErrorType {
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
    pub fn into_parts(self) -> (CreateGuildErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for CreateGuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateGuildErrorType::NameInvalid { .. } => f.write_str("the guild name is invalid"),
            CreateGuildErrorType::TooManyChannels { .. } => {
                f.write_str("too many channels were provided")
            }
            CreateGuildErrorType::TooManyRoles { .. } => {
                f.write_str("too many roles were provided")
            }
        }
    }
}

impl Error for CreateGuildError {}

/// Type of [`CreateGuildError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateGuildErrorType {
    /// The name of the guild is either fewer than 2 UTF-16 characters or more than 100 UTF-16
    /// characters.
    NameInvalid {
        /// Provided name.
        name: String,
    },
    /// The number of channels provided is too many.
    ///
    /// The maximum amount is 500.
    TooManyChannels {
        /// Provided channels.
        channels: Vec<GuildChannelFields>,
    },
    /// The number of roles provided is too many.
    ///
    /// The maximum amount is 250.
    TooManyRoles {
        /// Provided roles.
        roles: Vec<RoleFields>,
    },
}

#[derive(Serialize)]
struct CreateGuildFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    afk_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Vec<GuildChannelFields>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_message_notifications: Option<DefaultMessageNotificationLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explicit_content_filter: Option<ExplicitContentFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleFields>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_channel_flags: Option<SystemChannelFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_level: Option<VerificationLevel>,
}

/// Role fields sent to Discord.
///
/// Use [`RoleFieldsBuilder`] to build one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoleFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    pub id: RoleId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentionable: Option<bool>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
}

/// Variants of channel fields sent to Discord.
///
/// Use [`GuildChannelFieldsBuilder`] to build one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum GuildChannelFields {
    Category(CategoryFields),
    Text(TextFields),
    Voice(VoiceFields),
}

impl GuildChannelFields {
    pub const fn id(&self) -> ChannelId {
        match self {
            Self::Category(c) => c.id,
            Self::Text(t) => t.id,
            Self::Voice(v) => v.id,
        }
    }
}

/// Category channel fields sent to Discord.
///
/// Use [`CategoryFieldsBuilder`] to build one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CategoryFields {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
}

/// Text channel fields sent to Discord.
///
/// Use [`TextFieldsBuilder`] to build one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TextFields {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

/// Voice channel fields sent to Discord.
///
/// Use [`VoiceFieldsBuilder`] to build one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VoiceFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u64>,
}

/// Create a new request to create a guild.
///
/// The minimum length of the name is 2 UTF-16 characters and the maximum is 100 UTF-16 characters.
/// This endpoint can only be used by bots in less than 10 guilds.
pub struct CreateGuild<'a> {
    fields: CreateGuildFields,
    http: &'a Client,
}

impl<'a> CreateGuild<'a> {
    pub(crate) fn new(http: &'a Client, name: String) -> Result<Self, CreateGuildError> {
        if !validate::guild_name(&name) {
            return Err(CreateGuildError {
                kind: CreateGuildErrorType::NameInvalid { name },
            });
        }

        Ok(Self {
            fields: CreateGuildFields {
                afk_channel_id: None,
                afk_timeout: None,
                channels: None,
                default_message_notifications: None,
                explicit_content_filter: None,
                icon: None,
                name,
                roles: None,
                system_channel_id: None,
                system_channel_flags: None,
                verification_level: None,
            },
            http,
        })
    }

    /// Add a role to the list of roles.
    pub fn add_role(mut self, role: RoleFields) -> Self {
        if self.fields.roles.is_none() {
            let builder = RoleFieldsBuilder::new("@everyone".to_owned());
            self.fields.roles.replace(vec![builder.build()]);
        }

        if let Some(roles) = self.fields.roles.as_mut() {
            roles.push(role);
        }

        self
    }

    /// Set the ID of the AFK voice channel.
    ///
    /// This must be an ID specified in [`channels`].
    ///
    /// [`channels`]: Self::channels
    pub const fn afk_channel_id(mut self, afk_channel_id: ChannelId) -> Self {
        self.fields.afk_channel_id = Some(afk_channel_id);

        self
    }

    /// Set the AFK timeout, in seconds.
    pub const fn afk_timeout(mut self, afk_timeout: u64) -> Self {
        self.fields.afk_timeout = Some(afk_timeout);

        self
    }

    /// Set the channels to create with the guild.
    ///
    /// The maximum number of channels that can be provided is 500.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twilight_http::{
    ///     Client,
    ///     request::guild::create_guild::{
    ///         GuildChannelFieldsBuilder, CategoryFieldsBuilder, TextFieldsBuilder,
    ///         VoiceFieldsBuilder,
    ///     },
    /// };
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token".to_owned());
    ///
    /// let text = TextFieldsBuilder::new("text channel".to_owned())?.build();
    /// let voice = VoiceFieldsBuilder::new("voice channel".to_owned())?.build();
    /// let text2 = TextFieldsBuilder::new("other text channel".to_owned())?
    ///     .topic("posting".to_owned())?
    ///     .build();
    ///
    /// let category = CategoryFieldsBuilder::new("category channel".to_owned())?
    ///     .add_text(text2)
    ///     .add_voice(voice);
    ///
    /// let channels = GuildChannelFieldsBuilder::new()
    ///     .add_text(text)
    ///     .add_category_builder(category)
    ///     .build();
    ///
    /// let guild = client.create_guild("guild name".to_owned())?
    ///     .channels(channels)?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildErrorType::TooManyChannels`] error type if the
    /// number of channels is over 500.
    pub fn channels(mut self, channels: Vec<GuildChannelFields>) -> Result<Self, CreateGuildError> {
        // Error 30013
        // <https://discordapp.com/developers/docs/topics/opcodes-and-status-codes#json>
        if channels.len() > 500 {
            return Err(CreateGuildError {
                kind: CreateGuildErrorType::TooManyChannels { channels },
            });
        }

        self.fields.channels.replace(channels);

        Ok(self)
    }

    /// Set the default message notification level. Refer to [the discord docs] for more
    /// information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#create-guild
    pub const fn default_message_notifications(
        mut self,
        default_message_notifications: DefaultMessageNotificationLevel,
    ) -> Self {
        self.fields.default_message_notifications = Some(default_message_notifications);

        self
    }

    /// Set the explicit content filter level.
    pub const fn explicit_content_filter(
        mut self,
        explicit_content_filter: ExplicitContentFilter,
    ) -> Self {
        self.fields.explicit_content_filter = Some(explicit_content_filter);

        self
    }

    /// Set the icon.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn icon(mut self, icon: String) -> Self {
        self.fields.icon.replace(icon);

        self
    }

    /// Override the everyone role of the guild.
    ///
    /// If there are not yet roles set with [`roles`], this will create a role override in the
    /// first position. Discord understands the first role in the list to override @everyone.
    /// If there are roles, this replaces the first role in the position.
    ///
    /// [`roles`]: Self::roles
    pub fn override_everyone(mut self, everyone: RoleFields) -> Self {
        if let Some(roles) = self.fields.roles.as_mut() {
            roles.remove(0);
            roles.insert(0, everyone);
        } else {
            self.fields.roles.replace(vec![everyone]);
        }

        self
    }

    /// Set the channel where system messages will be posted.
    ///
    /// This must be an ID specified in [`channels`].
    ///
    /// [`channels`]: Self::channels
    pub const fn system_channel_id(mut self, system_channel_id: ChannelId) -> Self {
        self.fields.system_channel_id = Some(system_channel_id);

        self
    }

    /// Set the guild's [`SystemChannelFlags`].
    pub const fn system_channel_flags(mut self, system_channel_flags: SystemChannelFlags) -> Self {
        self.fields.system_channel_flags = Some(system_channel_flags);

        self
    }

    /// Set the roles to create with the guild.
    ///
    /// The maximum number of roles that can be provided is 250.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twilight_http::{Client, request::guild::create_guild::RoleFieldsBuilder};
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token".to_owned());
    ///
    /// let roles = vec![RoleFieldsBuilder::new("role 1".to_owned()).color(0x543923)?.build()];
    /// client.create_guild("guild name".to_owned())?.roles(roles)?.exec().await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildErrorType::TooManyRoles`] error type if the
    /// number of roles is over 250.
    pub fn roles(mut self, mut roles: Vec<RoleFields>) -> Result<Self, CreateGuildError> {
        if roles.len() > 250 {
            return Err(CreateGuildError {
                kind: CreateGuildErrorType::TooManyRoles { roles },
            });
        }

        if let Some(prev_roles) = self.fields.roles.as_mut() {
            roles.insert(0, prev_roles.remove(0));
        } else {
            let builder = RoleFieldsBuilder::new("@everyone".to_owned());
            roles.insert(0, builder.build());
        }

        self.fields.roles.replace(roles);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<PartialGuild> {
        let mut request = Request::builder(Route::CreateGuild);

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
