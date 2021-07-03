mod builder;

pub use self::builder::ClientBuilder;

use crate::{
    error::{Error, ErrorType},
    ratelimiting::Ratelimiter,
    request::{
        application::{
            CreateFollowupMessage, CreateGlobalCommand, CreateGuildCommand, DeleteFollowupMessage,
            DeleteGlobalCommand, DeleteGuildCommand, DeleteOriginalResponse, GetCommandPermissions,
            GetGlobalCommands, GetGuildCommandPermissions, GetGuildCommands, InteractionCallback,
            InteractionError, InteractionErrorType, SetCommandPermissions, SetGlobalCommands,
            SetGuildCommands, UpdateCommandPermissions, UpdateFollowupMessage, UpdateGlobalCommand,
            UpdateGuildCommand, UpdateOriginalResponse,
        },
        channel::stage::create_stage_instance::CreateStageInstanceError,
        guild::{
            create_guild::CreateGuildError, create_guild_channel::CreateGuildChannelError,
            update_guild_channel_positions::Position,
        },
        prelude::*,
        GetUserApplicationInfo, Method, Request,
    },
    response::ResponseFuture,
    API_VERSION,
};
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    Body,
};
use std::{
    convert::TryFrom,
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time;
use twilight_model::{
    application::{
        callback::InteractionResponse,
        command::{permissions::CommandPermissions, Command},
    },
    channel::message::allowed_mentions::AllowedMentions,
    guild::Permissions,
    id::{
        ApplicationId, ChannelId, CommandId, EmojiId, GuildId, IntegrationId, InteractionId,
        MessageId, RoleId, UserId, WebhookId,
    },
};

#[cfg(feature = "hyper-rustls")]
type HttpsConnector<T> = hyper_rustls::HttpsConnector<T>;
#[cfg(all(feature = "hyper-tls", not(feature = "hyper-rustls")))]
type HttpsConnector<T> = hyper_tls::HttpsConnector<T>;

struct State {
    http: HyperClient<HttpsConnector<HttpConnector>, Body>,
    default_headers: Option<HeaderMap>,
    proxy: Option<Box<str>>,
    ratelimiter: Option<Ratelimiter>,
    timeout: Duration,
    token_invalid: Arc<AtomicBool>,
    token: Option<Box<str>>,
    use_http: bool,
    pub(crate) application_id: AtomicU64,
    pub(crate) default_allowed_mentions: Option<AllowedMentions>,
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("State")
            .field("http", &self.http)
            .field("default_headers", &self.default_headers)
            .field("proxy", &self.proxy)
            .field("ratelimiter", &self.ratelimiter)
            .field("token", &self.token)
            .field("use_http", &self.use_http)
            .finish()
    }
}

/// Twilight's http client.
///
/// Almost all of the client methods require authentication, and as such, the client must be
/// supplied with a Discord Token. Get yours [here].
///
/// # OAuth
///
/// To use Bearer tokens prefix the token with `"Bearer "`, including the space
/// at the end like so:
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
///
/// let bearer = env::var("BEARER_TOKEN")?;
/// let token = format!("Bearer {}", bearer);
///
/// let client = Client::new(token);
/// # Ok(()) }
/// ```
///
/// # Cloning
///
/// The client internally wraps its data within an Arc. This means that the
/// client can be cloned and passed around tasks and threads cheaply.
///
/// # Unauthorized behavior
///
/// When the client encounters an Unauthorized response it will take note that
/// the configured token is invalid. This may occur when the token has been
/// revoked or expired. When this happens, you must create a new client with the
/// new token. The client will no longer execute requests in order to
/// prevent API bans and will always return [`ErrorType::Unauthorized`].
///
/// # Examples
///
/// Create a client called `client`:
/// ```rust,no_run
/// use twilight_http::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
/// # Ok(()) }
/// ```
///
/// Use [`ClientBuilder`] to create a client called `client`, with a shorter timeout:
/// ```rust,no_run
/// use twilight_http::Client;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::builder()
///     .token("my token")
///     .timeout(Duration::from_secs(5))
///     .build();
/// # Ok(()) }
/// ```
///
/// All the examples on this page assume you have already created a client, and have named it
/// `client`.
///
/// [here]: https://discord.com/developers/applications
#[derive(Clone, Debug)]
pub struct Client {
    state: Arc<State>,
}

impl Client {
    /// Create a new `hyper-rustls` or `hyper-tls` backed client with a token.
    #[cfg_attr(docsrs, doc(cfg(any(feature = "hyper-rustls", feature = "hyper-tls"))))]
    pub fn new(token: impl Into<String>) -> Self {
        ClientBuilder::default().token(token).build()
    }

    /// Create a new builder to create a client.
    ///
    /// Refer to its documentation for more information.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Retrieve an immutable reference to the token used by the client.
    ///
    /// If the initial token provided is not prefixed with `Bot `, it will be, and this method
    /// reflects that.
    pub fn token(&self) -> Option<&str> {
        self.state.token.as_deref()
    }

    /// Retrieve the [`ApplicationId`] used by interaction methods.
    pub fn application_id(&self) -> Option<ApplicationId> {
        let id = self.state.application_id.load(Ordering::Relaxed);

        if id != 0 {
            return Some(ApplicationId(id));
        }

        None
    }

    /// Set a new [`ApplicationId`] after building the client.
    ///
    /// Returns the previous ID, if there was one.
    pub fn set_application_id(&self, application_id: ApplicationId) -> Option<ApplicationId> {
        let prev = self
            .state
            .application_id
            .swap(application_id.0, Ordering::Relaxed);

        if prev != 0 {
            return Some(ApplicationId(prev));
        }

        None
    }

    /// Get the default [`AllowedMentions`] for sent messages.
    pub fn default_allowed_mentions(&self) -> Option<AllowedMentions> {
        self.state.default_allowed_mentions.clone()
    }

    /// Get the Ratelimiter used by the client internally.
    ///
    /// This will return `None` only if ratelimit handling
    /// has been explicitly disabled in the [`ClientBuilder`].
    pub fn ratelimiter(&self) -> Option<Ratelimiter> {
        self.state.ratelimiter.clone()
    }

    /// Get the audit log for a guild.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::GuildId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// let guild_id = GuildId(101);
    /// let audit_log = client
    /// // not done
    ///     .audit_log(guild_id)
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn audit_log(&self, guild_id: GuildId) -> GetAuditLog<'_> {
        GetAuditLog::new(self, guild_id)
    }

    /// Retrieve the bans for a guild.
    ///
    /// # Examples
    ///
    /// Retrieve the bans for guild `1`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::GuildId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(1);
    ///
    /// let bans = client.bans(guild_id).exec().await?;
    /// # Ok(()) }
    /// ```
    pub const fn bans(&self, guild_id: GuildId) -> GetBans<'_> {
        GetBans::new(self, guild_id)
    }

    /// Get information about a ban of a guild.
    ///
    /// Includes the user banned and the reason.
    pub const fn ban(&self, guild_id: GuildId, user_id: UserId) -> GetBan<'_> {
        GetBan::new(self, guild_id, user_id)
    }

    /// Bans a user from a guild, optionally with the number of days' worth of
    /// messages to delete and the reason.
    ///
    /// # Examples
    ///
    /// Ban user `200` from guild `100`, deleting
    /// 1 day's worth of messages, for the reason `"memes"`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::{request::AuditLogReason, Client};
    /// use twilight_model::id::{GuildId, UserId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(100);
    /// let user_id = UserId(200);
    /// client.create_ban(guild_id, user_id)
    ///     .delete_message_days(1)?
    ///     .reason("memes")?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn create_ban(&self, guild_id: GuildId, user_id: UserId) -> CreateBan<'_> {
        CreateBan::new(self, guild_id, user_id)
    }

    /// Remove a ban from a user in a guild.
    ///
    /// # Examples
    ///
    /// Unban user `200` from guild `100`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::{GuildId, UserId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(100);
    /// let user_id = UserId(200);
    ///
    /// client.delete_ban(guild_id, user_id).exec().await?;
    /// # Ok(()) }
    /// ```
    pub const fn delete_ban(&self, guild_id: GuildId, user_id: UserId) -> DeleteBan<'_> {
        DeleteBan::new(self, guild_id, user_id)
    }

    /// Get a channel by its ID.
    ///
    /// # Examples
    ///
    /// Get channel `100`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::ChannelId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let channel_id = ChannelId(100);
    /// #
    /// let channel = client.channel(channel_id).exec().await?;
    /// # Ok(()) }
    /// ```
    pub const fn channel(&self, channel_id: ChannelId) -> GetChannel<'_> {
        GetChannel::new(self, channel_id)
    }

    /// Delete a channel by ID.
    pub const fn delete_channel(&self, channel_id: ChannelId) -> DeleteChannel<'_> {
        DeleteChannel::new(self, channel_id)
    }

    /// Update a channel.
    ///
    /// All fields are optional. The minimum length of the name is 2 UTF-16 characters and the
    /// maximum is 100 UTF-16 characters.
    pub fn update_channel(&self, channel_id: ChannelId) -> UpdateChannel<'_> {
        UpdateChannel::new(self, channel_id)
    }

    /// Follows a news channel by [`ChannelId`].
    ///
    /// The type returned is [`FollowedChannel`].
    ///
    /// [`FollowedChannel`]: ::twilight_model::channel::FollowedChannel
    pub const fn follow_news_channel(
        &self,
        channel_id: ChannelId,
        webhook_channel_id: ChannelId,
    ) -> FollowNewsChannel<'_> {
        FollowNewsChannel::new(self, channel_id, webhook_channel_id)
    }

    /// Get the invites for a guild channel.
    ///
    /// Requires the [`MANAGE_CHANNELS`] permission. This method only works if
    /// the channel is of type [`GuildChannel`].
    ///
    /// [`MANAGE_CHANNELS`]: twilight_model::guild::Permissions::MANAGE_CHANNELS
    /// [`GuildChannel`]: twilight_model::channel::GuildChannel
    pub const fn channel_invites(&self, channel_id: ChannelId) -> GetChannelInvites<'_> {
        GetChannelInvites::new(self, channel_id)
    }

    /// Get channel messages, by [`ChannelId`].
    ///
    /// Only one of [`after`], [`around`], and [`before`] can be specified at a time.
    /// Once these are specified, the type returned is [`GetChannelMessagesConfigured`].
    ///
    /// If [`limit`] is unspecified, the default set by Discord is 50.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use twilight_http::Client;
    /// use twilight_model::id::{ChannelId, MessageId};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("my token");
    /// let channel_id = ChannelId(123);
    /// let message_id = MessageId(234);
    /// let limit: u64 = 6;
    ///
    /// let messages = client
    ///     .channel_messages(channel_id)
    ///     .before(message_id)
    ///     .limit(limit)?
    ///     .exec()
    ///     .await?;
    ///
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`GetChannelMessagesErrorType::LimitInvalid`] error type if
    /// the amount is less than 1 or greater than 100.
    ///
    /// [`after`]: GetChannelMessages::after
    /// [`around`]: GetChannelMessages::around
    /// [`before`]: GetChannelMessages::before
    /// [`GetChannelMessagesConfigured`]: crate::request::channel::message::GetChannelMessagesConfigured
    /// [`limit`]: GetChannelMessages::limit
    /// [`GetChannelMessagesErrorType::LimitInvalid`]: crate::request::channel::message::get_channel_messages::GetChannelMessagesErrorType::LimitInvalid
    pub fn channel_messages(&self, channel_id: ChannelId) -> GetChannelMessages<'_> {
        GetChannelMessages::new(self, channel_id)
    }

    pub const fn delete_channel_permission(
        &self,
        channel_id: ChannelId,
    ) -> DeleteChannelPermission<'_> {
        DeleteChannelPermission::new(self, channel_id)
    }

    /// Update the permissions for a role or a user in a channel.
    ///
    /// # Examples:
    ///
    /// Create permission overrides for a role to view the channel, but not send messages:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::guild::Permissions;
    /// use twilight_model::id::{ChannelId, RoleId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    ///
    /// let channel_id = ChannelId(123);
    /// let allow = Permissions::VIEW_CHANNEL;
    /// let deny = Permissions::SEND_MESSAGES;
    /// let role_id = RoleId(432);
    ///
    /// client.update_channel_permission(channel_id, allow, deny)
    ///     .role(role_id)
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub const fn update_channel_permission(
        &self,
        channel_id: ChannelId,
        allow: Permissions,
        deny: Permissions,
    ) -> UpdateChannelPermission<'_> {
        UpdateChannelPermission::new(self, channel_id, allow, deny)
    }

    /// Get all the webhooks of a channel.
    pub const fn channel_webhooks(&self, channel_id: ChannelId) -> GetChannelWebhooks<'_> {
        GetChannelWebhooks::new(self, channel_id)
    }

    /// Get information about the current user.
    pub const fn current_user(&self) -> GetCurrentUser<'_> {
        GetCurrentUser::new(self)
    }

    /// Get information about the current bot application.
    pub const fn current_user_application(&self) -> GetUserApplicationInfo<'_> {
        GetUserApplicationInfo::new(self)
    }

    /// Update the current user.
    ///
    /// All paramaters are optional. If the username is changed, it may cause the discriminator to
    /// be randomized.
    pub fn update_current_user(&self) -> UpdateCurrentUser<'_> {
        UpdateCurrentUser::new(self)
    }

    /// Update the current user's voice state.
    ///
    /// All paramaters are optional.
    ///
    /// # Caveats
    ///
    /// - `channel_id` must currently point to a stage channel.
    /// - Current user must have already joined `channel_id`.
    pub const fn update_current_user_voice_state(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> UpdateCurrentUserVoiceState<'_> {
        UpdateCurrentUserVoiceState::new(self, guild_id, channel_id)
    }

    /// Get the current user's connections.
    ///
    /// Requires the `connections` `OAuth2` scope.
    pub const fn current_user_connections(&self) -> GetCurrentUserConnections<'_> {
        GetCurrentUserConnections::new(self)
    }

    /// Returns a list of guilds for the current user.
    ///
    /// # Examples
    ///
    /// Get the first 25 guilds with an ID after `300` and before
    /// `400`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::GuildId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let after = GuildId(300);
    /// let before = GuildId(400);
    /// let guilds = client.current_user_guilds()
    ///     .after(after)
    ///     .before(before)
    ///     .limit(25)?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub const fn current_user_guilds(&self) -> GetCurrentUserGuilds<'_> {
        GetCurrentUserGuilds::new(self)
    }

    /// Changes the user's nickname in a guild.
    pub fn update_current_user_nick(
        &self,
        guild_id: GuildId,
        nick: impl Into<String>,
    ) -> UpdateCurrentUserNick<'_> {
        UpdateCurrentUserNick::new(self, guild_id, nick)
    }

    /// Get the emojis for a guild, by the guild's id.
    ///
    /// # Examples
    ///
    /// Get the emojis for guild `100`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::GuildId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(100);
    ///
    /// client.emojis(guild_id).exec().await?;
    /// # Ok(()) }
    /// ```
    pub const fn emojis(&self, guild_id: GuildId) -> GetEmojis<'_> {
        GetEmojis::new(self, guild_id)
    }

    /// Get an emoji for a guild by the the guild's ID and emoji's ID.
    ///
    /// # Examples
    ///
    /// Get emoji `100` from guild `50`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::{EmojiId, GuildId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(50);
    /// let emoji_id = EmojiId(100);
    ///
    /// client.emoji(guild_id, emoji_id).exec().await?;
    /// # Ok(()) }
    /// ```
    pub const fn emoji(&self, guild_id: GuildId, emoji_id: EmojiId) -> GetEmoji<'_> {
        GetEmoji::new(self, guild_id, emoji_id)
    }

    /// Create an emoji in a guild.
    ///
    /// The emoji must be a Data URI, in the form of `data:image/{type};base64,{data}` where
    /// `{type}` is the image MIME type and `{data}` is the base64-encoded image.  Refer to [the
    /// discord docs] for more information about image data.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn create_emoji(
        &self,
        guild_id: GuildId,
        name: impl Into<String>,
        image: impl Into<String>,
    ) -> CreateEmoji<'_> {
        CreateEmoji::new(self, guild_id, name, image)
    }

    /// Delete an emoji in a guild, by id.
    pub const fn delete_emoji(&self, guild_id: GuildId, emoji_id: EmojiId) -> DeleteEmoji<'_> {
        DeleteEmoji::new(self, guild_id, emoji_id)
    }

    /// Update an emoji in a guild, by id.
    pub fn update_emoji(&self, guild_id: GuildId, emoji_id: EmojiId) -> UpdateEmoji<'_> {
        UpdateEmoji::new(self, guild_id, emoji_id)
    }

    /// Get information about the gateway, optionally with additional information detailing the
    /// number of shards to use and sessions remaining.
    ///
    /// # Examples
    ///
    /// Get the gateway connection URL without bot information:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let info = client.gateway().exec().await?;
    /// # Ok(()) }
    /// ```
    ///
    /// Get the gateway connection URL with additional shard and session information, which
    /// requires specifying a bot token:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let info = client.gateway().authed().exec().await?.model().await?;
    ///
    /// println!("URL: {}", info.url);
    /// println!("Recommended shards to use: {}", info.shards);
    /// # Ok(()) }
    /// ```
    pub const fn gateway(&self) -> GetGateway<'_> {
        GetGateway::new(self)
    }

    /// Get information about a guild.
    pub fn guild(&self, guild_id: GuildId) -> GetGuild<'_> {
        GetGuild::new(self, guild_id)
    }

    /// Create a new request to create a guild.
    ///
    /// The minimum length of the name is 2 UTF-16 characters and the maximum is 100 UTF-16
    /// characters. This endpoint can only be used by bots in less than 10 guilds.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildErrorType::NameInvalid`] error type if the name
    /// length is too short or too long.
    ///
    /// [`CreateGuildErrorType::NameInvalid`]: crate::request::guild::create_guild::CreateGuildErrorType::NameInvalid
    pub fn create_guild(
        &self,
        name: impl Into<String>,
    ) -> Result<CreateGuild<'_>, CreateGuildError> {
        CreateGuild::new(self, name)
    }

    /// Delete a guild permanently. The user must be the owner.
    pub const fn delete_guild(&self, guild_id: GuildId) -> DeleteGuild<'_> {
        DeleteGuild::new(self, guild_id)
    }

    /// Update a guild.
    ///
    /// All endpoints are optional. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild
    pub fn update_guild(&self, guild_id: GuildId) -> UpdateGuild<'_> {
        UpdateGuild::new(self, guild_id)
    }

    /// Leave a guild by id.
    pub const fn leave_guild(&self, guild_id: GuildId) -> LeaveGuild<'_> {
        LeaveGuild::new(self, guild_id)
    }

    /// Get the channels in a guild.
    pub const fn guild_channels(&self, guild_id: GuildId) -> GetGuildChannels<'_> {
        GetGuildChannels::new(self, guild_id)
    }

    /// Create a new request to create a guild channel.
    ///
    /// All fields are optional except for name. The minimum length of the name is 2 UTF-16
    /// characters and the maximum is 100 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildChannelErrorType::NameInvalid`] error type when
    /// the length of the name is either fewer than 2 UTF-16 characters or more than 100 UTF-16 characters.
    ///
    /// Returns a [`CreateGuildChannelErrorType::RateLimitPerUserInvalid`] error
    /// type when the seconds of the rate limit per user is more than 21600.
    ///
    /// Returns a [`CreateGuildChannelErrorType::TopicInvalid`] error type when
    /// the length of the topic is more than 1024 UTF-16 characters.
    ///
    /// [`CreateGuildChannelErrorType::NameInvalid`]: crate::request::guild::create_guild_channel::CreateGuildChannelErrorType::NameInvalid
    /// [`CreateGuildChannelErrorType::RateLimitPerUserInvalid`]: crate::request::guild::create_guild_channel::CreateGuildChannelErrorType::RateLimitPerUserInvalid
    /// [`CreateGuildChannelErrorType::TopicInvalid`]: crate::request::guild::create_guild_channel::CreateGuildChannelErrorType::TopicInvalid
    pub fn create_guild_channel(
        &self,
        guild_id: GuildId,
        name: impl Into<String>,
    ) -> Result<CreateGuildChannel<'_>, CreateGuildChannelError> {
        CreateGuildChannel::new(self, guild_id, name)
    }

    /// Modify the positions of the channels.
    ///
    /// The minimum amount of channels to modify, is a swap between two channels.
    ///
    /// This function accepts an `Iterator` of `(ChannelId, u64)`. It also
    /// accepts an `Iterator` of `Position`, which has extra fields.
    pub fn update_guild_channel_positions(
        &self,
        guild_id: GuildId,
        channel_positions: impl Iterator<Item = impl Into<Position>>,
    ) -> UpdateGuildChannelPositions<'_> {
        UpdateGuildChannelPositions::new(self, guild_id, channel_positions)
    }

    /// Get the guild widget.
    ///
    /// Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#get-guild-widget
    pub const fn guild_widget(&self, guild_id: GuildId) -> GetGuildWidget<'_> {
        GetGuildWidget::new(self, guild_id)
    }

    /// Modify the guild widget.
    pub fn update_guild_widget(&self, guild_id: GuildId) -> UpdateGuildWidget<'_> {
        UpdateGuildWidget::new(self, guild_id)
    }

    /// Get the guild's integrations.
    pub const fn guild_integrations(&self, guild_id: GuildId) -> GetGuildIntegrations<'_> {
        GetGuildIntegrations::new(self, guild_id)
    }

    /// Delete an integration for a guild, by the integration's id.
    pub const fn delete_guild_integration(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
    ) -> DeleteGuildIntegration<'_> {
        DeleteGuildIntegration::new(self, guild_id, integration_id)
    }

    /// Get information about the invites of a guild.
    ///
    /// Requires the [`MANAGE_GUILD`] permission.
    ///
    /// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
    pub const fn guild_invites(&self, guild_id: GuildId) -> GetGuildInvites<'_> {
        GetGuildInvites::new(self, guild_id)
    }

    /// Get the members of a guild, by id.
    ///
    /// The upper limit to this request is 1000. If more than 1000 members are needed, the requests
    /// must be chained. Discord defaults the limit to 1.
    ///
    /// # Examples
    ///
    /// Get the first 500 members of guild `100` after user ID `3000`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::{GuildId, UserId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(100);
    /// let user_id = UserId(3000);
    /// let members = client.guild_members(guild_id).after(user_id).exec().await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`GetGuildMembersErrorType::LimitInvalid`] error type if the
    /// limit is invalid.
    ///
    /// [`GetGuildMembersErrorType::LimitInvalid`]: crate::request::guild::member::get_guild_members::GetGuildMembersErrorType::LimitInvalid
    pub fn guild_members(&self, guild_id: GuildId) -> GetGuildMembers<'_> {
        GetGuildMembers::new(self, guild_id)
    }

    /// Search the members of a specific guild by a query.
    ///
    /// The upper limit to this request is 1000. Discord defaults the limit to 1.
    ///
    /// # Examples
    ///
    /// Get the first 10 members of guild `100` matching `Wumpus`:
    ///
    /// ```rust,no_run
    /// use twilight_http::Client;
    /// use twilight_model::id::GuildId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("my token");
    ///
    /// let guild_id = GuildId(100);
    /// let members = client.search_guild_members(guild_id, String::from("Wumpus"))
    ///     .limit(10)?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`SearchGuildMembersErrorType::LimitInvalid`] error type if
    /// the limit is invalid.
    ///
    /// [`GUILD_MEMBERS`]: ../../twilight_model/gateway/struct.Intents.html#associatedconstant.GUILD_MEMBERS
    /// [`SearchGuildMembersErrorType::LimitInvalid`]: ../request/guild/member/search_guild_members/enum.SearchGuildMembersError.html#variant.LimitInvalid
    pub fn search_guild_members(
        &self,
        guild_id: GuildId,
        query: impl Into<String>,
    ) -> SearchGuildMembers<'_> {
        SearchGuildMembers::new(self, guild_id, query)
    }

    /// Get a member of a guild, by their id.
    pub const fn guild_member(&self, guild_id: GuildId, user_id: UserId) -> GetMember<'_> {
        GetMember::new(self, guild_id, user_id)
    }

    /// Add a user to a guild.
    ///
    /// An access token for the user with `guilds.join` scope is required. All
    /// other fields are optional. Refer to [the discord docs] for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns [`AddGuildMemberErrorType::NicknameInvalid`] if the nickname is
    /// too short or too long.
    ///
    /// [`AddGuildMemberErrorType::NickNameInvalid`]: crate::request::guild::member::add_guild_member::AddGuildMemberErrorType::NicknameInvalid
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#add-guild-member
    pub fn add_guild_member(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        access_token: impl Into<String>,
    ) -> AddGuildMember<'_> {
        AddGuildMember::new(self, guild_id, user_id, access_token)
    }

    /// Kick a member from a guild.
    pub const fn remove_guild_member(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> RemoveMember<'_> {
        RemoveMember::new(self, guild_id, user_id)
    }

    /// Update a guild member.
    ///
    /// All fields are optional. Refer to [the discord docs] for more information.
    ///
    /// # Examples
    ///
    /// Update a member's nickname to "pinky pie" and server mute them:
    ///
    /// ```rust,no_run
    /// use std::env;
    /// use twilight_http::Client;
    /// use twilight_model::id::{GuildId, UserId};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// let member = client.update_guild_member(GuildId(1), UserId(2))
    ///     .mute(true)
    ///     .nick(Some("pinkie pie".to_owned()))?
    ///     .exec()
    ///     .await?
    ///     .model()
    ///     .await?;
    ///
    /// println!("user {} now has the nickname '{:?}'", member.user.id, member.nick);
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`UpdateGuildMemberErrorType::NicknameInvalid`] if the nickname length is too short or too
    /// long.
    ///
    /// [`UpdateGuildMemberErrorType::NicknameInvalid`]: crate::request::guild::member::update_guild_member::UpdateGuildMemberErrorType::NicknameInvalid
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild-member
    pub fn update_guild_member(&self, guild_id: GuildId, user_id: UserId) -> UpdateGuildMember<'_> {
        UpdateGuildMember::new(self, guild_id, user_id)
    }

    /// Add a role to a member in a guild.
    ///
    /// # Examples
    ///
    /// In guild `1`, add role `2` to user `3`, for the reason `"test"`:
    ///
    /// ```rust,no_run
    /// # use twilight_http::{request::AuditLogReason, Client};
    /// use twilight_model::id::{GuildId, RoleId, UserId};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let guild_id = GuildId(1);
    /// let role_id = RoleId(2);
    /// let user_id = UserId(3);
    ///
    /// client.add_guild_member_role(guild_id, user_id, role_id)
    ///     .reason("test")?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn add_guild_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> AddRoleToMember<'_> {
        AddRoleToMember::new(self, guild_id, user_id, role_id)
    }

    /// Remove a role from a member in a guild, by id.
    pub fn remove_guild_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> RemoveRoleFromMember<'_> {
        RemoveRoleFromMember::new(self, guild_id, user_id, role_id)
    }

    /// For public guilds, get the guild preview.
    ///
    /// This works even if the user is not in the guild.
    pub const fn guild_preview(&self, guild_id: GuildId) -> GetGuildPreview<'_> {
        GetGuildPreview::new(self, guild_id)
    }

    /// Get the counts of guild members to be pruned.
    pub fn guild_prune_count(&self, guild_id: GuildId) -> GetGuildPruneCount<'_> {
        GetGuildPruneCount::new(self, guild_id)
    }

    /// Begin a guild prune.
    ///
    /// Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/guild#begin-guild-prune
    pub fn create_guild_prune(&self, guild_id: GuildId) -> CreateGuildPrune<'_> {
        CreateGuildPrune::new(self, guild_id)
    }

    /// Get a guild's vanity url, if there is one.
    pub const fn guild_vanity_url(&self, guild_id: GuildId) -> GetGuildVanityUrl<'_> {
        GetGuildVanityUrl::new(self, guild_id)
    }

    /// Get voice region data for the guild.
    ///
    /// Can return VIP servers if the guild is VIP-enabled.
    pub const fn guild_voice_regions(&self, guild_id: GuildId) -> GetGuildVoiceRegions<'_> {
        GetGuildVoiceRegions::new(self, guild_id)
    }

    /// Get the webhooks of a guild.
    pub const fn guild_webhooks(&self, guild_id: GuildId) -> GetGuildWebhooks<'_> {
        GetGuildWebhooks::new(self, guild_id)
    }

    /// Get the guild's welcome screen.
    pub const fn guild_welcome_screen(&self, guild_id: GuildId) -> GetGuildWelcomeScreen<'_> {
        GetGuildWelcomeScreen::new(self, guild_id)
    }

    /// Update the guild's welcome screen.
    ///
    /// Requires the [`MANAGE_GUILD`] permission.
    ///
    /// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
    pub fn update_guild_welcome_screen(&self, guild_id: GuildId) -> UpdateGuildWelcomeScreen<'_> {
        UpdateGuildWelcomeScreen::new(self, guild_id)
    }

    /// Get information about an invite by its code.
    ///
    /// If [`with_counts`] is called, the returned invite will contain
    /// approximate member counts.  If [`with_expiration`] is called, it will
    /// contain the expiration date.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let invite = client
    ///     .invite("code")
    ///     .with_counts()
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`with_counts`]: crate::request::channel::invite::GetInvite::with_counts
    /// [`with_expiration`]: crate::request::channel::invite::GetInvite::with_expiration
    pub fn invite(&self, code: impl Into<String>) -> GetInvite<'_> {
        GetInvite::new(self, code)
    }

    /// Create an invite, with options.
    ///
    /// Requires the [`CREATE_INVITE`] permission.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::ChannelId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let channel_id = ChannelId(123);
    /// let invite = client
    ///     .create_invite(channel_id)
    ///     .max_uses(3)?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`CREATE_INVITE`]: twilight_model::guild::Permissions::CREATE_INVITE
    pub fn create_invite(&self, channel_id: ChannelId) -> CreateInvite<'_> {
        CreateInvite::new(self, channel_id)
    }

    /// Delete an invite by its code.
    ///
    /// Requires the [`MANAGE_CHANNELS`] permission on the channel this invite
    /// belongs to, or [`MANAGE_GUILD`] to remove any invite across the guild.
    ///
    /// [`MANAGE_CHANNELS`]: twilight_model::guild::Permissions::MANAGE_CHANNELS
    /// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
    pub fn delete_invite(&self, code: impl Into<String>) -> DeleteInvite<'_> {
        DeleteInvite::new(self, code)
    }

    /// Get a message by [`ChannelId`] and [`MessageId`].
    pub const fn message(&self, channel_id: ChannelId, message_id: MessageId) -> GetMessage<'_> {
        GetMessage::new(self, channel_id, message_id)
    }

    /// Send a message to a channel.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::ChannelId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let channel_id = ChannelId(123);
    /// let message = client
    ///     .create_message(channel_id)
    ///     .content("Twilight is best pony")?
    ///     .tts(true)
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// The method [`content`] returns
    /// [`CreateMessageErrorType::ContentInvalid`] if the content is over 2000
    /// UTF-16 characters.
    ///
    /// The method [`embed`] returns
    /// [`CreateMessageErrorType::EmbedTooLarge`] if the length of the embed
    /// is over 6000 characters.
    ///
    /// [`content`]: crate::request::channel::message::create_message::CreateMessage::content
    /// [`embed`]: crate::request::channel::message::create_message::CreateMessage::embed
    /// [`CreateMessageErrorType::ContentInvalid`]:
    /// crate::request::channel::message::create_message::CreateMessageErrorType::ContentInvalid
    /// [`CreateMessageErrorType::EmbedTooLarge`]:
    /// crate::request::channel::message::create_message::CreateMessageErrorType::EmbedTooLarge
    pub fn create_message(&self, channel_id: ChannelId) -> CreateMessage<'_> {
        CreateMessage::new(self, channel_id)
    }

    /// Delete a message by [`ChannelId`] and [`MessageId`].
    pub const fn delete_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> DeleteMessage<'_> {
        DeleteMessage::new(self, channel_id, message_id)
    }

    /// Delete messages by [`ChannelId`] and Vec<[`MessageId`]>.
    ///
    /// The vec count can be between 2 and 100. If the supplied [`MessageId`]s are invalid, they
    /// still count towards the lower and upper limits. This method will not delete messages older
    /// than two weeks. Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#bulk-delete-messages
    pub fn delete_messages(
        &self,
        channel_id: ChannelId,
        message_ids: impl Into<Vec<MessageId>>,
    ) -> DeleteMessages<'_> {
        DeleteMessages::new(self, channel_id, message_ids)
    }

    /// Update a message by [`ChannelId`] and [`MessageId`].
    ///
    /// You can pass `None` to any of the methods to remove the associated field.
    /// For example, if you have a message with an embed you want to remove, you can
    /// use `.[embed](None)` to remove the embed.
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
    /// let client = Client::new("my token");
    /// client.update_message(ChannelId(1), MessageId(2))
    ///     .content("test update".to_owned())?
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
    /// # let client = Client::new("my token");
    /// client.update_message(ChannelId(1), MessageId(2))
    ///     .content(None)?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [embed]: Self::embed
    pub fn update_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> UpdateMessage<'_> {
        UpdateMessage::new(self, channel_id, message_id)
    }

    /// Crosspost a message by [`ChannelId`] and [`MessageId`].
    pub const fn crosspost_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> CrosspostMessage<'_> {
        CrosspostMessage::new(self, channel_id, message_id)
    }

    /// Get the pins of a channel.
    pub const fn pins(&self, channel_id: ChannelId) -> GetPins<'_> {
        GetPins::new(self, channel_id)
    }

    /// Create a new pin in a channel, by ID.
    pub const fn create_pin(&self, channel_id: ChannelId, message_id: MessageId) -> CreatePin<'_> {
        CreatePin::new(self, channel_id, message_id)
    }

    /// Delete a pin in a channel, by ID.
    pub const fn delete_pin(&self, channel_id: ChannelId, message_id: MessageId) -> DeletePin<'_> {
        DeletePin::new(self, channel_id, message_id)
    }

    /// Get a list of users that reacted to a message with an `emoji`.
    ///
    /// This endpoint is limited to 100 users maximum, so if a message has more than 100 reactions,
    /// requests must be chained until all reactions are retireved.
    pub fn reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
    ) -> GetReactions<'_> {
        GetReactions::new(self, channel_id, message_id, emoji)
    }

    /// Create a reaction in a [`ChannelId`] on a [`MessageId`].
    ///
    /// The reaction must be a variant of [`RequestReactionType`].
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use twilight_http::{Client, request::channel::reaction::RequestReactionType};
    /// # use twilight_model::{
    /// #     id::{ChannelId, MessageId},
    /// # };
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// #
    /// let channel_id = ChannelId(123);
    /// let message_id = MessageId(456);
    /// let emoji = RequestReactionType::Unicode { name: String::from("🌃") };
    ///
    /// let reaction = client
    ///     .create_reaction(channel_id, message_id, emoji)
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub const fn create_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
    ) -> CreateReaction<'_> {
        CreateReaction::new(self, channel_id, message_id, emoji)
    }

    /// Delete the current user's (`@me`) reaction on a message.
    pub fn delete_current_user_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
    ) -> DeleteReaction<'_> {
        DeleteReaction::new(self, channel_id, message_id, emoji, "@me")
    }

    /// Delete a reaction by a user on a message.
    pub fn delete_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
        user_id: UserId,
    ) -> DeleteReaction<'_> {
        DeleteReaction::new(self, channel_id, message_id, emoji, user_id.to_string())
    }

    /// Remove all reactions on a message of an emoji.
    pub const fn delete_all_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RequestReactionType,
    ) -> DeleteAllReaction<'_> {
        DeleteAllReaction::new(self, channel_id, message_id, emoji)
    }

    /// Delete all reactions by all users on a message.
    pub const fn delete_all_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> DeleteAllReactions<'_> {
        DeleteAllReactions::new(self, channel_id, message_id)
    }

    /// Fire a Typing Start event in the channel.
    pub const fn create_typing_trigger(&self, channel_id: ChannelId) -> CreateTypingTrigger<'_> {
        CreateTypingTrigger::new(self, channel_id)
    }

    /// Create a group DM.
    ///
    /// This endpoint is limited to 10 active group DMs.
    pub const fn create_private_channel(&self, recipient_id: UserId) -> CreatePrivateChannel<'_> {
        CreatePrivateChannel::new(self, recipient_id)
    }

    /// Get the roles of a guild.
    pub const fn roles(&self, guild_id: GuildId) -> GetGuildRoles<'_> {
        GetGuildRoles::new(self, guild_id)
    }

    /// Create a role in a guild.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::GuildId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// let guild_id = GuildId(234);
    ///
    /// client.create_role(guild_id)
    ///     .color(0xd90083)
    ///     .name("Bright Pink")
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn create_role(&self, guild_id: GuildId) -> CreateRole<'_> {
        CreateRole::new(self, guild_id)
    }

    /// Delete a role in a guild, by id.
    pub const fn delete_role(&self, guild_id: GuildId, role_id: RoleId) -> DeleteRole<'_> {
        DeleteRole::new(self, guild_id, role_id)
    }

    /// Update a role by guild id and its id.
    pub fn update_role(&self, guild_id: GuildId, role_id: RoleId) -> UpdateRole<'_> {
        UpdateRole::new(self, guild_id, role_id)
    }

    /// Modify the position of the roles.
    ///
    /// The minimum amount of roles to modify, is a swap between two roles.
    pub fn update_role_positions(
        &self,
        guild_id: GuildId,
        roles: impl Iterator<Item = (RoleId, u64)>,
    ) -> UpdateRolePositions<'_> {
        UpdateRolePositions::new(self, guild_id, roles)
    }

    /// Create a new stage instance associated with a stage channel.
    ///
    /// Requires the user to be a moderator of the stage channel.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateStageInstanceError`] of type [`InvalidTopic`] when the
    /// topic is not between 1 and 120 characters in length.
    ///
    /// [`InvalidTopic`]: crate::request::channel::stage::create_stage_instance::CreateStageInstanceErrorType::InvalidTopic
    pub fn create_stage_instance(
        &self,
        channel_id: ChannelId,
        topic: impl Into<String>,
    ) -> Result<CreateStageInstance<'_>, CreateStageInstanceError> {
        CreateStageInstance::new(self, channel_id, topic)
    }

    /// Gets the stage instance associated with a stage channel, if it exists.
    pub const fn stage_instance(&self, channel_id: ChannelId) -> GetStageInstance<'_> {
        GetStageInstance::new(self, channel_id)
    }

    /// Update fields of an existing stage instance.
    ///
    /// Requires the user to be a moderator of the stage channel.
    pub fn update_stage_instance(&self, channel_id: ChannelId) -> UpdateStageInstance<'_> {
        UpdateStageInstance::new(self, channel_id)
    }

    /// Delete the stage instance of a stage channel.
    ///
    /// Requires the user to be a moderator of the stage channel.
    pub const fn delete_stage_instance(&self, channel_id: ChannelId) -> DeleteStageInstance<'_> {
        DeleteStageInstance::new(self, channel_id)
    }

    /// Create a new guild based on a template.
    ///
    /// This endpoint can only be used by bots in less than 10 guilds.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateGuildFromTemplateErrorType::NameInvalid`] error type
    /// if the name is invalid.
    ///
    /// [`CreateGuildFromTemplateErrorType::NameInvalid`]: crate::request::template::create_guild_from_template::CreateGuildFromTemplateErrorType::NameInvalid
    pub fn create_guild_from_template(
        &self,
        template_code: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<CreateGuildFromTemplate<'_>, CreateGuildFromTemplateError> {
        CreateGuildFromTemplate::new(self, template_code, name)
    }

    /// Create a template from the current state of the guild.
    ///
    /// Requires the `MANAGE_GUILD` permission. The name must be at least 1 and
    /// at most 100 characters in length.
    ///
    /// # Errors
    ///
    /// Returns a [`CreateTemplateErrorType::NameInvalid`] error type if the
    /// name is invalid.
    ///
    /// [`CreateTemplateErrorType::NameInvalid`]: crate::request::template::create_template::CreateTemplateErrorType::NameInvalid
    pub fn create_template(
        &self,
        guild_id: GuildId,
        name: impl Into<String>,
    ) -> Result<CreateTemplate<'_>, CreateTemplateError> {
        CreateTemplate::new(self, guild_id, name)
    }

    /// Delete a template by ID and code.
    pub fn delete_template(
        &self,
        guild_id: GuildId,
        template_code: impl Into<String>,
    ) -> DeleteTemplate<'_> {
        DeleteTemplate::new(self, guild_id, template_code)
    }

    /// Get a template by its code.
    pub fn get_template(&self, template_code: impl Into<String>) -> GetTemplate<'_> {
        GetTemplate::new(self, template_code)
    }

    /// Get a list of templates in a guild, by ID.
    pub const fn get_templates(&self, guild_id: GuildId) -> GetTemplates<'_> {
        GetTemplates::new(self, guild_id)
    }

    /// Sync a template to the current state of the guild, by ID and code.
    pub fn sync_template(
        &self,
        guild_id: GuildId,
        template_code: impl Into<String>,
    ) -> SyncTemplate<'_> {
        SyncTemplate::new(self, guild_id, template_code)
    }

    /// Update the template's metadata, by ID and code.
    pub fn update_template(
        &self,
        guild_id: GuildId,
        template_code: impl Into<String>,
    ) -> UpdateTemplate<'_> {
        UpdateTemplate::new(self, guild_id, template_code)
    }

    /// Get a user's information by id.
    pub fn user(&self, user_id: UserId) -> GetUser<'_> {
        GetUser::new(self, user_id.to_string())
    }

    /// Update another user's voice state.
    ///
    /// # Caveats
    ///
    /// - `channel_id` must currently point to a stage channel.
    /// - User must already have joined `channel_id`.
    pub const fn update_user_voice_state(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> UpdateUserVoiceState<'_> {
        UpdateUserVoiceState::new(self, guild_id, user_id, channel_id)
    }

    /// Get a list of voice regions that can be used when creating a guild.
    pub const fn voice_regions(&self) -> GetVoiceRegions<'_> {
        GetVoiceRegions::new(self)
    }

    /// Get a webhook by ID.
    pub fn webhook(&self, id: WebhookId) -> GetWebhook<'_> {
        GetWebhook::new(self, id)
    }

    /// Create a webhook in a channel.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::ChannelId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// let channel_id = ChannelId(123);
    ///
    /// let webhook = client
    ///     .create_webhook(channel_id, "Twily Bot")
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn create_webhook(
        &self,
        channel_id: ChannelId,
        name: impl Into<String>,
    ) -> CreateWebhook<'_> {
        CreateWebhook::new(self, channel_id, name)
    }

    /// Delete a webhook by its ID.
    pub const fn delete_webhook(&self, id: WebhookId) -> DeleteWebhook<'_> {
        DeleteWebhook::new(self, id)
    }

    /// Update a webhook by ID.
    pub fn update_webhook(&self, webhook_id: WebhookId) -> UpdateWebhook<'_> {
        UpdateWebhook::new(self, webhook_id)
    }

    /// Update a webhook, with a token, by ID.
    pub fn update_webhook_with_token(
        &self,
        webhook_id: WebhookId,
        token: impl Into<String>,
    ) -> UpdateWebhookWithToken<'_> {
        UpdateWebhookWithToken::new(self, webhook_id, token)
    }

    /// Executes a webhook, sending a message to its channel.
    ///
    /// You can only specify one of [`content`], [`embeds`], or [`file`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use twilight_http::Client;
    /// # use twilight_model::id::WebhookId;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("my token");
    /// let id = WebhookId(432);
    /// #
    /// let webhook = client
    ///     .execute_webhook(id, "webhook token")
    ///     .content("Pinkie...")
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// [`content`]: crate::request::channel::webhook::ExecuteWebhook::content
    /// [`embeds`]: crate::request::channel::webhook::ExecuteWebhook::embeds
    /// [`file`]: crate::request::channel::webhook::ExecuteWebhook::file
    pub fn execute_webhook(
        &self,
        webhook_id: WebhookId,
        token: impl Into<String>,
    ) -> ExecuteWebhook<'_> {
        ExecuteWebhook::new(self, webhook_id, token)
    }

    /// Get a webhook message by [`WebhookId`], token, and [`MessageId`].
    ///
    /// [`WebhookId`]: twilight_model::id::WebhookId
    /// [`MessageId`]: twilight_model::id::MessageId
    pub fn webhook_message(
        &self,
        webhook_id: WebhookId,
        token: impl Into<String>,
        message_id: MessageId,
    ) -> GetWebhookMessage<'_> {
        GetWebhookMessage::new(self, webhook_id, token, message_id)
    }

    /// Update a message executed by a webhook.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::{MessageId, WebhookId};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// client.update_webhook_message(WebhookId(1), "token here", MessageId(2))
    ///     .content(Some("new message content".to_owned()))?
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn update_webhook_message(
        &self,
        webhook_id: WebhookId,
        token: impl Into<String>,
        message_id: MessageId,
    ) -> UpdateWebhookMessage<'_> {
        UpdateWebhookMessage::new(self, webhook_id, token, message_id)
    }

    /// Delete a message executed by a webhook.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use twilight_http::Client;
    /// use twilight_model::id::{MessageId, WebhookId};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::new("token");
    /// client
    ///     .delete_webhook_message(WebhookId(1), "token here", MessageId(2))
    ///     .exec()
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub fn delete_webhook_message(
        &self,
        webhook_id: WebhookId,
        token: impl Into<String>,
        message_id: MessageId,
    ) -> DeleteWebhookMessage<'_> {
        DeleteWebhookMessage::new(self, webhook_id, token, message_id)
    }

    /// Respond to an interaction, by ID and token.
    pub fn interaction_callback(
        &self,
        interaction_id: InteractionId,
        interaction_token: impl Into<String>,
        response: InteractionResponse,
    ) -> InteractionCallback<'_> {
        InteractionCallback::new(self, interaction_id, interaction_token, response)
    }

    /// Edit the original message, by its token.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn update_interaction_original(
        &self,
        interaction_token: impl Into<String>,
    ) -> Result<UpdateOriginalResponse<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(UpdateOriginalResponse::new(
            self,
            application_id,
            interaction_token,
        ))
    }

    /// Delete the original message, by its token.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn delete_interaction_original(
        &self,
        interaction_token: impl Into<String>,
    ) -> Result<DeleteOriginalResponse<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(DeleteOriginalResponse::new(
            self,
            application_id,
            interaction_token,
        ))
    }

    /// Create a followup message, by an interaction token.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn create_followup_message(
        &self,
        interaction_token: impl Into<String>,
    ) -> Result<CreateFollowupMessage<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(CreateFollowupMessage::new(
            self,
            application_id,
            interaction_token,
        ))
    }

    /// Edit a followup message, by an interaction token.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn update_followup_message(
        &self,
        interaction_token: impl Into<String>,
        message_id: MessageId,
    ) -> Result<UpdateFollowupMessage<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(UpdateFollowupMessage::new(
            self,
            application_id,
            interaction_token,
            message_id,
        ))
    }

    /// Delete a followup message by interaction token and the message's ID.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn delete_followup_message(
        &self,
        interaction_token: impl Into<String>,
        message_id: MessageId,
    ) -> Result<DeleteFollowupMessage<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(DeleteFollowupMessage::new(
            self,
            application_id,
            interaction_token,
            message_id,
        ))
    }

    /// Create a new command in a guild.
    ///
    /// The name must be between 3 and 32 characters in length, and the
    /// description must be between 1 and 100 characters in length. Creating a
    /// guild command with the same name as an already-existing guild command in
    /// the same guild will overwrite the old command. See [the discord docs]
    /// for more information.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    ///
    /// Returns an [`InteractionErrorType::CommandNameValidationFailed`]
    /// error type if the command name is not between 3 and 32 characters.
    ///
    /// Returns an [`InteractionErrorType::CommandDescriptionValidationFailed`]
    /// error type if the command description is not between 1 and
    /// 100 characters.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#create-guild-application-command
    pub fn create_guild_command(
        &self,
        guild_id: GuildId,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<CreateGuildCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        CreateGuildCommand::new(&self, application_id, guild_id, name, description)
    }

    /// Fetch all commands for a guild, by ID.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn get_guild_commands(
        &self,
        guild_id: GuildId,
    ) -> Result<GetGuildCommands<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(GetGuildCommands::new(self, application_id, guild_id))
    }

    /// Edit a command in a guild, by ID.
    ///
    /// You must specify a name and description. See [the discord docs] for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    ///
    /// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#edit-guild-application-command
    pub fn update_guild_command(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<UpdateGuildCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(UpdateGuildCommand::new(
            self,
            application_id,
            guild_id,
            command_id,
        ))
    }

    /// Delete a command in a guild, by ID.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn delete_guild_command(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<DeleteGuildCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(DeleteGuildCommand::new(
            self,
            application_id,
            guild_id,
            command_id,
        ))
    }

    /// Set a guild's commands.
    ///
    /// This method is idempotent: it can be used on every start, without being
    /// ratelimited if there aren't changes to the commands.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn set_guild_commands(
        &self,
        guild_id: GuildId,
        commands: Vec<Command>,
    ) -> Result<SetGuildCommands<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(SetGuildCommands::new(
            self,
            application_id,
            guild_id,
            commands,
        ))
    }

    /// Create a new global command.
    ///
    /// The name must be between 3 and 32 characters in length, and the
    /// description must be between 1 and 100 characters in length. Creating a
    /// command with the same name as an already-existing global command will
    /// overwrite the old command. See [the discord docs] for more information.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    ///
    /// Returns an [`InteractionErrorType::CommandNameValidationFailed`]
    /// error type if the command name is not between 3 and 32 characters.
    ///
    /// Returns an [`InteractionErrorType::CommandDescriptionValidationFailed`]
    /// error type if the command description is not between 1 and
    /// 100 characters.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#create-global-application-command
    pub fn create_global_command(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<CreateGlobalCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        CreateGlobalCommand::new(self, application_id, name, description)
    }

    /// Fetch all global commands for your application.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn get_global_commands(&self) -> Result<GetGlobalCommands<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(GetGlobalCommands::new(self, application_id))
    }

    /// Edit a global command, by ID.
    ///
    /// You must specify a name and description. See [the discord docs] for more
    /// information.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    ///
    /// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#edit-global-application-command
    pub fn update_global_command(
        &self,
        command_id: CommandId,
    ) -> Result<UpdateGlobalCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(UpdateGlobalCommand::new(self, application_id, command_id))
    }

    /// Delete a global command, by ID.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn delete_global_command(
        &self,
        command_id: CommandId,
    ) -> Result<DeleteGlobalCommand<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(DeleteGlobalCommand::new(self, application_id, command_id))
    }

    /// Set global commands.
    ///
    /// This method is idempotent: it can be used on every start, without being
    /// ratelimited if there aren't changes to the commands.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn set_global_commands(
        &self,
        commands: Vec<Command>,
    ) -> Result<SetGlobalCommands<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(SetGlobalCommands::new(self, application_id, commands))
    }

    /// Fetch command permissions for a command from the current application
    /// in a guild.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn get_command_permissions(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<GetCommandPermissions<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(GetCommandPermissions::new(
            &self,
            application_id,
            guild_id,
            command_id,
        ))
    }

    /// Fetch command permissions for all commands from the current
    /// application in a guild.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn get_guild_command_permissions(
        &self,
        guild_id: GuildId,
    ) -> Result<GetGuildCommandPermissions<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        Ok(GetGuildCommandPermissions::new(
            self,
            application_id,
            guild_id,
        ))
    }

    /// Update command permissions for a single command in a guild.
    ///
    /// This overwrites the command permissions so the full set of permissions
    /// have to be sent every time.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn update_command_permissions(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
        permissions: Vec<CommandPermissions>,
    ) -> Result<UpdateCommandPermissions<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        UpdateCommandPermissions::new(self, application_id, guild_id, command_id, permissions)
    }

    /// Update command permissions for all commands in a guild.
    ///
    /// This overwrites the command permissions so the full set of permissions
    /// have to be sent every time.
    ///
    /// # Errors
    ///
    /// Returns an [`InteractionErrorType::ApplicationIdNotPresent`]
    /// error type if an application ID has not been configured via
    /// [`Client::set_application_id`].
    pub fn set_command_permissions(
        &self,
        guild_id: GuildId,
        permissions: impl Iterator<Item = (CommandId, CommandPermissions)>,
    ) -> Result<SetCommandPermissions<'_>, InteractionError> {
        let application_id = self.application_id().ok_or(InteractionError {
            kind: InteractionErrorType::ApplicationIdNotPresent,
        })?;

        SetCommandPermissions::new(self, application_id, guild_id, permissions)
    }

    /// Execute a request, returning a future resolving to a [`Response`].
    ///
    /// # Errors
    ///
    /// Returns an [`ErrorType::Unauthorized`] error type if the configured
    /// token has become invalid due to expiration, revokation, etc.
    ///
    /// [`Response`]: super::response::Response
    pub fn request<T>(&self, request: Request) -> ResponseFuture<T> {
        match self.try_request::<T>(request) {
            Ok(future) => future,
            Err(source) => ResponseFuture::error(source),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn try_request<T>(&self, request: Request) -> Result<ResponseFuture<T>, Error> {
        if self.state.token_invalid.load(Ordering::Relaxed) {
            return Err(Error {
                kind: ErrorType::Unauthorized,
                source: None,
            });
        }

        let Request {
            body,
            form,
            headers: req_headers,
            method,
            path: bucket,
            path_str: path,
            use_authorization_token,
        } = request;

        let protocol = if self.state.use_http { "http" } else { "https" };
        let host = self.state.proxy.as_deref().unwrap_or("discord.com");

        let url = format!("{}://{}/api/v{}/{}", protocol, host, API_VERSION, path);
        #[cfg(feature = "tracing")]
        tracing::debug!("URL: {:?}", url);

        let mut builder = hyper::Request::builder()
            .method(method.into_hyper())
            .uri(&url);

        if use_authorization_token {
            if let Some(ref token) = self.state.token {
                let value = HeaderValue::from_str(&token).map_err(|source| {
                    #[allow(clippy::borrow_interior_mutable_const)]
                    let name = AUTHORIZATION.to_string();

                    Error {
                        kind: ErrorType::CreatingHeader { name },
                        source: Some(Box::new(source)),
                    }
                })?;

                if let Some(headers) = builder.headers_mut() {
                    headers.insert(AUTHORIZATION, value);
                }
            }
        }

        let user_agent = HeaderValue::from_static(concat!(
            "DiscordBot (",
            env!("CARGO_PKG_HOMEPAGE"),
            ", ",
            env!("CARGO_PKG_VERSION"),
            ") Twilight-rs",
        ));

        if let Some(headers) = builder.headers_mut() {
            if let Some(form) = &form {
                if let Ok(content_type) = HeaderValue::try_from(form.content_type()) {
                    headers.insert(CONTENT_TYPE, content_type);
                }
            } else if let Some(bytes) = &body {
                let len = bytes.len();
                headers.insert(CONTENT_LENGTH, len.into());

                let content_type = HeaderValue::from_static("application/json");
                headers.insert(CONTENT_TYPE, content_type);
            }

            headers.insert(USER_AGENT, user_agent);

            if let Some(req_headers) = req_headers {
                for (maybe_name, value) in req_headers {
                    if let Some(name) = maybe_name {
                        headers.insert(name, value);
                    }
                }
            }

            if let Some(default_headers) = &self.state.default_headers {
                for (name, value) in default_headers {
                    headers.insert(name, HeaderValue::from(value));
                }
            }
        }

        let req = if let Some(form) = form {
            let form_bytes = form.build();
            if let Some(headers) = builder.headers_mut() {
                headers.insert(CONTENT_LENGTH, form_bytes.len().into());
            };
            builder
                .body(Body::from(form_bytes))
                .map_err(|source| Error {
                    kind: ErrorType::BuildingRequest,
                    source: Some(Box::new(source)),
                })?
        } else if let Some(bytes) = body {
            builder.body(Body::from(bytes)).map_err(|source| Error {
                kind: ErrorType::BuildingRequest,
                source: Some(Box::new(source)),
            })?
        } else if method == Method::Put || method == Method::Post || method == Method::Patch {
            if let Some(headers) = builder.headers_mut() {
                headers.insert(CONTENT_LENGTH, 0.into());
            }

            builder.body(Body::empty()).map_err(|source| Error {
                kind: ErrorType::BuildingRequest,
                source: Some(Box::new(source)),
            })?
        } else {
            builder.body(Body::empty()).map_err(|source| Error {
                kind: ErrorType::BuildingRequest,
                source: Some(Box::new(source)),
            })?
        };

        let inner = self.state.http.request(req);
        let token_invalid = Arc::clone(&self.state.token_invalid);

        // Clippy suggests bad code; an `Option::map_or_else` won't work here
        // due to move semantics in both cases.
        #[allow(clippy::option_if_let_else)]
        if let Some(ratelimiter) = self.state.ratelimiter.as_ref() {
            let rx = ratelimiter.ticket(bucket);

            Ok(ResponseFuture::ratelimit(
                None,
                token_invalid,
                rx,
                self.state.timeout,
                inner,
            ))
        } else {
            Ok(ResponseFuture::new(
                token_invalid,
                time::timeout(self.state.timeout, inner),
                None,
            ))
        }
    }
}
