use super::{path::Path, route_display::RouteDisplay};
use crate::request::{channel::reaction::RequestReactionType, Method};
use twilight_model::id::RoleId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Route<'a> {
    /// Route information to add a user to a guild.
    AddGuildMember { guild_id: u64, user_id: u64 },
    /// Route information to add a role to guild member.
    AddMemberRole {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the role.
        role_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to create a ban on a user in a guild.
    CreateBan {
        /// The number of days' worth of the user's messages to delete in the
        /// guild's channels.
        delete_message_days: Option<u64>,
        /// The ID of the guild.
        guild_id: u64,
        /// The reason for the ban.
        reason: Option<&'a str>,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to create a channel in a guild.
    CreateChannel {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create an emoji in a guild.
    CreateEmoji {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create a global command.
    CreateGlobalCommand {
        /// The ID of the owner application.
        application_id: u64,
    },
    /// Route information to create a guild.
    CreateGuild,
    /// Route information to create a guild command.
    CreateGuildCommand {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create a guild from a template.
    CreateGuildFromTemplate {
        /// Code of the template.
        template_code: &'a str,
    },
    /// Route information to create a guild's integration.
    CreateGuildIntegration {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create a prune in a guild.
    CreateGuildPrune {
        /// Whether to compute the number of pruned users.
        compute_prune_count: Option<bool>,
        /// The number of days that a user must be offline before being able to
        /// be pruned.
        days: Option<u64>,
        /// The ID of the guild.
        guild_id: u64,
        /// The roles to filter the prune by.
        ///
        /// A user must have at least one of these roles to be able to be
        /// pruned.
        include_roles: &'a [RoleId],
    },
    /// Route information to create an invite to a channel.
    CreateInvite {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to create a message in a channel.
    CreateMessage {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to create a private channel.
    CreatePrivateChannel,
    /// Route information to create a reaction on a message.
    CreateReaction {
        /// The ID of the channel.
        channel_id: u64,
        /// The URI encoded custom or unicode emoji.
        emoji: &'a RequestReactionType<'a>,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to create a role in a guild.
    CreateRole {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create a stage instance.
    CreateStageInstance,
    /// Route information to create a guild template.
    CreateTemplate {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to create a typing trigger in a channel.
    CreateTypingTrigger {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to create a webhook in a channel.
    CreateWebhook {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to crosspost a message to following guilds.
    CrosspostMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to delete a ban on a user in a guild.
    DeleteBan {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to delete a channel.
    DeleteChannel {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to delete a guild's custom emoji.
    DeleteEmoji {
        /// The ID of the emoji.
        emoji_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to delete a global command.
    DeleteGlobalCommand {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
    },
    /// Route information to delete a guild.
    DeleteGuild {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to delete a guild command.
    DeleteGuildCommand {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to delete a guild integration.
    DeleteGuildIntegration {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the integration.
        integration_id: u64,
    },
    /// Route information to delete an invite.
    DeleteInvite {
        /// The unique invite code.
        code: &'a str,
    },
    /// Route information to delete the original interaction response.
    DeleteInteractionOriginal {
        /// The ID of the owner application
        application_id: u64,
        /// The token of the interaction.
        interaction_token: &'a str,
    },
    /// Route information to delete a channel's message.
    DeleteMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to bulk delete messages in a channel.
    DeleteMessages {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to delete all of the reactions on a message.
    DeleteMessageReactions {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to delete all of the reactions on a message with a
    /// specific emoji.
    DeleteMessageSpecificReaction {
        /// The ID of the channel.
        channel_id: u64,
        /// The URI encoded custom or unicode emoji.
        emoji: &'a RequestReactionType<'a>,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to delete a permission overwrite for a role or user in
    /// a channel.
    DeletePermissionOverwrite {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the target role or user.
        target_id: u64,
    },
    /// Route information to delete the current user's reaction on a message.
    DeleteReactionCurrentUser {
        /// ID of the channel.
        channel_id: u64,
        /// URI encoded custom or unicode emoji.
        emoji: &'a RequestReactionType<'a>,
        /// ID of the message.
        message_id: u64,
    },
    /// Route information to delete a user's reaction on a message.
    DeleteReaction {
        /// The ID of the channel.
        channel_id: u64,
        /// The URI encoded custom or unicode emoji.
        emoji: &'a RequestReactionType<'a>,
        /// The ID of the message.
        message_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to delete a guild's role.
    DeleteRole {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the role.
        role_id: u64,
    },
    /// Route information to delete a stage instance.
    DeleteStageInstance {
        /// ID of the stage channel.
        channel_id: u64,
    },
    /// Route information to delete a guild template.
    DeleteTemplate {
        /// The ID of the guild.
        guild_id: u64,
        /// The target template code.
        template_code: &'a str,
    },
    /// Route information to delete a message created by a webhook.
    DeleteWebhookMessage {
        message_id: u64,
        token: &'a str,
        webhook_id: u64,
    },
    /// Route information to delete a webhook.
    DeleteWebhook {
        /// The token of the webhook.
        token: Option<&'a str>,
        /// The ID of the webhook.
        webhook_id: u64,
    },
    /// Route information to execute a webhook by ID and token.
    ExecuteWebhook {
        /// The token of the webhook.
        token: &'a str,
        /// Whether to wait for a message response.
        wait: Option<bool>,
        /// The ID of the webhook.
        webhook_id: u64,
    },
    /// Route information to follow a news channel.
    FollowNewsChannel {
        /// The ID of the channel to follow.
        channel_id: u64,
    },
    /// Route information to get a paginated list of audit logs in a guild.
    GetAuditLogs {
        /// The type of action to get audit logs for.
        action_type: Option<u64>,
        /// The maximum ID of audit logs to get.
        before: Option<u64>,
        /// The ID of the guild.
        guild_id: u64,
        /// The maximum number of audit logs to get.
        limit: Option<u64>,
        /// The ID of the user, if specified.
        user_id: Option<u64>,
    },
    /// Route information to get information about a single ban in a guild.
    GetBan {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to get a guild's bans.
    GetBans {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a channel.
    GetChannel {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to get a channel's invites.
    GetChannelInvites {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to get a channel's webhooks.
    GetChannelWebhooks {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to get a guild's channels.
    GetChannels {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get permissions of a specific guild command.
    GetCommandPermissions {
        /// The ID of the application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get info about application the current bot user belongs to
    GetCurrentUserApplicationInfo,
    /// Route information to get the current user.
    GetCurrentUser,
    /// Route information to get an emoji by ID within a guild.
    GetEmoji {
        /// The ID of the emoji.
        emoji_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's emojis.
    GetEmojis {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get basic gateway information.
    GetGateway,
    /// Route information to get gateway information tailored to the current
    /// user.
    GetGatewayBot,
    GetGlobalCommands {
        /// The ID of the owner application.
        application_id: u64,
    },
    /// Route information to get a guild.
    GetGuild {
        /// The ID of the guild.
        guild_id: u64,
        /// Whether to include approximate member and presence counts for the
        /// guild.
        with_counts: bool,
    },
    /// Route information to get permissions of all guild commands.
    GetGuildCommandPermissions {
        /// The ID of the application.
        application_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get guild commands.
    GetGuildCommands {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's widget.
    GetGuildWidget {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's integrations.
    GetGuildIntegrations {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's invites.
    GetGuildInvites {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's members.
    GetGuildMembers {
        /// The minimum ID of members to get.
        after: Option<u64>,
        /// The ID of the guild.
        guild_id: u64,
        /// The maximum number of members to get.
        limit: Option<u64>,
        /// Whether to get the members' presences.
        presences: Option<bool>,
    },
    /// Route information to get a guild's preview.
    GetGuildPreview {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get the number of members that would be pruned,
    /// filtering by inactivity and users with one of the provided roles.
    GetGuildPruneCount {
        /// The number of days that a user must be offline before being able to
        /// be pruned.
        days: Option<u64>,
        /// The ID of the guild.
        guild_id: u64,
        /// The roles to filter the prune by.
        ///
        /// A user must have at least one of these roles to be able to be
        /// pruned.
        include_roles: &'a [RoleId],
    },
    /// Route information to get a guild's roles.
    GetGuildRoles {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's vanity URL.
    GetGuildVanityUrl {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's available voice regions.
    GetGuildVoiceRegions {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's welcome screen.
    GetGuildWelcomeScreen {
        /// ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a guild's webhooks.
    GetGuildWebhooks {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a paginated list of guilds.
    GetGuilds {
        /// The minimum ID of guilds to get.
        after: Option<u64>,
        /// The maximum ID of guilds to get.
        before: Option<u64>,
        /// The maximum number of guilds to get.
        limit: Option<u64>,
    },
    /// Route information to get an invite.
    GetInvite {
        /// The unique invite code.
        code: &'a str,
        /// Whether to retrieve statistics about the invite.
        with_counts: bool,
    },
    /// Route information to get an invite with an expiration.
    GetInviteWithExpiration {
        /// The unique invite code.
        code: &'a str,
        /// Whether to retrieve statistics about the invite.
        with_counts: bool,
        /// Whether to retrieve the expiration date of the invite.
        with_expiration: bool,
    },
    /// Route information to get a member.
    GetMember {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to get a single message in a channel.
    GetMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to get a paginated list of messages in a channel.
    GetMessages {
        /// The minimum ID of messages to get.
        after: Option<u64>,
        /// The message ID to get the messages around.
        around: Option<u64>,
        /// The maximum ID of messages to get.
        before: Option<u64>,
        /// The ID of the channel.
        channel_id: u64,
        /// The maximum number of messages to get.
        limit: Option<u64>,
    },
    /// Route information to get a channel's pins.
    GetPins {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to get the users who reacted to a message with a
    /// specified emoji.
    GetReactionUsers {
        /// The minimum ID of users to get.
        after: Option<u64>,
        /// The ID of the channel.
        channel_id: u64,
        /// The URI encoded custom or unicode emoji.
        emoji: &'a RequestReactionType<'a>,
        /// The maximum number of users to retrieve.
        limit: Option<u64>,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to get a stage instance.
    GetStageInstance {
        /// ID of the stage channel.
        channel_id: u64,
    },
    /// Route information to get a template.
    GetTemplate {
        /// The template code.
        template_code: &'a str,
    },
    /// Route information to get a list of templates from a guild.
    GetTemplates {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to get a user.
    GetUser {
        /// ID of the target user.
        user_id: u64,
    },
    /// Route information to get the current user's connections.
    GetUserConnections,
    /// Route information to get the current user's private channels and groups.
    GetUserPrivateChannels,
    /// Route information to get a list of the voice regions.
    GetVoiceRegions,
    /// Route information to get a webhook by ID, optionally with a token if the
    /// current user doesn't have access to it.
    GetWebhook {
        /// The token of the webhook.
        token: Option<&'a str>,
        /// The ID of the webhook.
        webhook_id: u64,
    },
    /// Route information to get a previously-sent webhook message.
    GetWebhookMessage {
        /// ID of the message.
        message_id: u64,
        /// Token of the webhook.
        token: &'a str,
        /// ID of the webhook.
        webhook_id: u64,
    },
    /// Route information to respond to an interaction.
    InteractionCallback {
        /// The ID of the interaction.
        interaction_id: u64,
        /// The token for the interaction.
        interaction_token: &'a str,
    },
    /// Route information to leave the guild.
    LeaveGuild {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to pin a message to a channel.
    PinMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to remove a member from a guild.
    RemoveMember {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to remove a role from a member.
    RemoveMemberRole {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the role.
        role_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to search for members in a guild.
    SearchGuildMembers {
        /// ID of the guild to search in.
        guild_id: u64,
        /// Upper limit of members to query for.
        limit: Option<u64>,
        /// Query to search by.
        query: &'a str,
    },
    /// Route information to set permissions of commands in a guild.
    SetCommandPermissions {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to set global commands.
    SetGlobalCommands {
        /// The ID of the owner application.
        application_id: u64,
    },
    /// Route information to set guild commands.
    SetGuildCommands {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to sync a guild's integration.
    SyncGuildIntegration {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the integration.
        integration_id: u64,
    },
    /// Route information to sync a template.
    SyncTemplate {
        /// The ID of the guild.
        guild_id: u64,
        /// The template code.
        template_code: &'a str,
    },
    /// Route information to unpin a message from a channel.
    UnpinMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to update a channel, such as a guild channel or group.
    UpdateChannel {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// Route information to edit permissions of a command in a guild.
    UpdateCommandPermissions {
        /// The ID of the application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update the current user.
    UpdateCurrentUser,
    /// Route information to update the current user's voice state.
    UpdateCurrentUserVoiceState {
        /// ID of the guild.
        guild_id: u64,
    },
    /// Route information to update an emoji.
    UpdateEmoji {
        /// The ID of the emoji.
        emoji_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update a global command.
    UpdateGlobalCommand {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
    },
    /// Route information to update a guild.
    UpdateGuild {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update a guild channel.
    UpdateGuildChannels {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update a guild command.
    UpdateGuildCommand {
        /// The ID of the owner application.
        application_id: u64,
        /// The ID of the command.
        command_id: u64,
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update a guild's widget.
    UpdateGuildWidget {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update a guild's integration.
    UpdateGuildIntegration {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the integration.
        integration_id: u64,
    },
    /// Route information to update a guild's welcome screen.
    UpdateGuildWelcomeScreen {
        /// ID of the guild.
        guild_id: u64,
    },
    /// Update the original interaction response.
    UpdateInteractionOriginal {
        /// The ID of the owner application.
        application_id: u64,
        /// The token for the interaction.
        interaction_token: &'a str,
    },
    /// Route information to update a member.
    UpdateMember {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the user.
        user_id: u64,
    },
    /// Route information to update a message.
    UpdateMessage {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the message.
        message_id: u64,
    },
    /// Route information to update the current member's nickname.
    UpdateNickname {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update the permission overwrite of a role or user
    /// in a channel.
    UpdatePermissionOverwrite {
        /// The ID of the channel.
        channel_id: u64,
        /// The ID of the role or user.
        target_id: u64,
    },
    /// Route information to update a role.
    UpdateRole {
        /// The ID of the guild.
        guild_id: u64,
        /// The ID of the role.
        role_id: u64,
    },
    /// Route information to update the positions of roles.
    UpdateRolePositions {
        /// The ID of the guild.
        guild_id: u64,
    },
    /// Route information to update an existing stage instance.
    UpdateStageInstance {
        /// ID of the stage channel.
        channel_id: u64,
    },
    /// Route information to update a template.
    UpdateTemplate {
        /// The ID of the guild.
        guild_id: u64,
        /// The template code.
        template_code: &'a str,
    },
    /// Route information to update a user's voice state.
    UpdateUserVoiceState {
        /// ID of the guild.
        guild_id: u64,
        /// ID of the user.
        user_id: u64,
    },
    /// Route information to update a message created by a webhook.
    UpdateWebhookMessage {
        message_id: u64,
        token: &'a str,
        webhook_id: u64,
    },
    /// Route information to update a webhook.
    UpdateWebhook {
        /// The token of the webhook.
        token: Option<&'a str>,
        /// The ID of the webhook.
        webhook_id: u64,
    },
}

impl<'a> Route<'a> {
    /// Display formatter of the route portion of a URL.
    ///
    /// # Examples
    ///
    /// Create a formatted representation of the [`GetPins`] route:
    ///
    /// ```
    /// use twilight_http::routing::Route;
    ///
    /// let route = Route::GetPins {
    ///     channel_id: 123,
    /// };
    /// assert_eq!("channels/123/pins", route.display().to_string());
    /// ```
    ///
    /// Create a formatted representation of the [`GetInvite`] route, which
    /// includes a query parameter:
    ///
    /// ```
    /// use twilight_http::routing::Route;
    ///
    /// let route = Route::GetInvite {
    ///     code: "twilight-rs",
    ///     with_counts: true,
    /// };
    ///
    /// assert_eq!(
    ///     "invites/twilight-rs?with-counts=true",
    ///     route.display().to_string(),
    /// );
    /// ```
    ///
    /// [`GetInvite`]: Self::GetInvite
    /// [`GetPins`]: Self::GetPins
    pub const fn display(&'a self) -> RouteDisplay<'a> {
        RouteDisplay::new(self)
    }

    /// HTTP method of the route.
    ///
    /// # Examples
    ///
    /// Assert that the [`GetGuild`] route returns [`Method::Get`]:
    ///
    /// ```
    /// use twilight_http::{request::Method, routing::Route};
    ///
    /// let route = Route::GetGuild {
    ///     guild_id: 123,
    ///     with_counts: false,
    /// };
    ///
    /// assert_eq!(Method::Get, route.method());
    /// ```
    ///
    /// [`GetGuild`]: Self::GetGuild
    #[allow(clippy::too_many_lines)]
    pub const fn method(&self) -> Method {
        match self {
            Self::DeleteBan { .. }
            | Self::DeleteChannel { .. }
            | Self::DeleteEmoji { .. }
            | Self::DeleteGlobalCommand { .. }
            | Self::DeleteGuild { .. }
            | Self::DeleteGuildCommand { .. }
            | Self::DeleteGuildIntegration { .. }
            | Self::DeleteInteractionOriginal { .. }
            | Self::DeleteInvite { .. }
            | Self::DeleteMessageReactions { .. }
            | Self::DeleteMessageSpecificReaction { .. }
            | Self::DeleteMessage { .. }
            | Self::DeletePermissionOverwrite { .. }
            | Self::DeleteReactionCurrentUser { .. }
            | Self::DeleteReaction { .. }
            | Self::DeleteRole { .. }
            | Self::DeleteStageInstance { .. }
            | Self::DeleteTemplate { .. }
            | Self::DeleteWebhookMessage { .. }
            | Self::DeleteWebhook { .. }
            | Self::LeaveGuild { .. }
            | Self::RemoveMember { .. }
            | Self::RemoveMemberRole { .. }
            | Self::UnpinMessage { .. } => Method::Delete,
            Self::GetAuditLogs { .. }
            | Self::GetBan { .. }
            | Self::GetBans { .. }
            | Self::GetGatewayBot
            | Self::GetChannel { .. }
            | Self::GetChannelInvites { .. }
            | Self::GetChannelWebhooks { .. }
            | Self::GetChannels { .. }
            | Self::GetCommandPermissions { .. }
            | Self::GetCurrentUserApplicationInfo
            | Self::GetCurrentUser
            | Self::GetEmoji { .. }
            | Self::GetEmojis { .. }
            | Self::GetGateway
            | Self::GetGlobalCommands { .. }
            | Self::GetGuild { .. }
            | Self::GetGuildCommandPermissions { .. }
            | Self::GetGuildCommands { .. }
            | Self::GetGuildIntegrations { .. }
            | Self::GetGuildInvites { .. }
            | Self::GetGuildMembers { .. }
            | Self::GetGuildPreview { .. }
            | Self::GetGuildPruneCount { .. }
            | Self::GetGuildRoles { .. }
            | Self::GetGuildVanityUrl { .. }
            | Self::GetGuildVoiceRegions { .. }
            | Self::GetGuildWelcomeScreen { .. }
            | Self::GetGuildWebhooks { .. }
            | Self::GetGuildWidget { .. }
            | Self::GetGuilds { .. }
            | Self::GetInvite { .. }
            | Self::GetInviteWithExpiration { .. }
            | Self::GetMember { .. }
            | Self::GetMessage { .. }
            | Self::GetMessages { .. }
            | Self::GetPins { .. }
            | Self::GetReactionUsers { .. }
            | Self::GetStageInstance { .. }
            | Self::GetTemplate { .. }
            | Self::GetTemplates { .. }
            | Self::GetUserConnections
            | Self::GetUserPrivateChannels
            | Self::GetUser { .. }
            | Self::GetVoiceRegions
            | Self::GetWebhook { .. }
            | Self::GetWebhookMessage { .. }
            | Self::SearchGuildMembers { .. } => Method::Get,
            Self::UpdateChannel { .. }
            | Self::UpdateCurrentUser
            | Self::UpdateCurrentUserVoiceState { .. }
            | Self::UpdateEmoji { .. }
            | Self::UpdateGlobalCommand { .. }
            | Self::UpdateGuild { .. }
            | Self::UpdateGuildChannels { .. }
            | Self::UpdateGuildCommand { .. }
            | Self::UpdateGuildWidget { .. }
            | Self::UpdateGuildIntegration { .. }
            | Self::UpdateGuildWelcomeScreen { .. }
            | Self::UpdateInteractionOriginal { .. }
            | Self::UpdateMember { .. }
            | Self::UpdateMessage { .. }
            | Self::UpdateNickname { .. }
            | Self::UpdateRole { .. }
            | Self::UpdateRolePositions { .. }
            | Self::UpdateStageInstance { .. }
            | Self::UpdateTemplate { .. }
            | Self::UpdateUserVoiceState { .. }
            | Self::UpdateWebhookMessage { .. }
            | Self::UpdateWebhook { .. } => Method::Patch,
            Self::CreateChannel { .. }
            | Self::CreateGlobalCommand { .. }
            | Self::CreateGuildCommand { .. }
            | Self::CreateEmoji { .. }
            | Self::CreateGuild
            | Self::CreateGuildFromTemplate { .. }
            | Self::CreateGuildIntegration { .. }
            | Self::CreateGuildPrune { .. }
            | Self::CreateInvite { .. }
            | Self::CreateMessage { .. }
            | Self::CreatePrivateChannel
            | Self::CreateRole { .. }
            | Self::CreateStageInstance { .. }
            | Self::CreateTemplate { .. }
            | Self::CreateTypingTrigger { .. }
            | Self::CreateWebhook { .. }
            | Self::CrosspostMessage { .. }
            | Self::DeleteMessages { .. }
            | Self::ExecuteWebhook { .. }
            | Self::FollowNewsChannel { .. }
            | Self::InteractionCallback { .. }
            | Self::SyncGuildIntegration { .. } => Method::Post,
            Self::AddGuildMember { .. }
            | Self::AddMemberRole { .. }
            | Self::CreateBan { .. }
            | Self::CreateReaction { .. }
            | Self::PinMessage { .. }
            | Self::SetCommandPermissions { .. }
            | Self::SetGlobalCommands { .. }
            | Self::SetGuildCommands { .. }
            | Self::SyncTemplate { .. }
            | Self::UpdateCommandPermissions { .. }
            | Self::UpdatePermissionOverwrite { .. } => Method::Put,
        }
    }

    /// Typed path of the route.
    ///
    /// Paths are used with the [`Ratelimiter`].
    ///
    /// # Examples
    ///
    /// Use a route's path to retrieve a ratelimiter ticket:
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use twilight_http::{ratelimiting::Ratelimiter, routing::Route};
    ///
    /// let ratelimiter = Ratelimiter::new();
    /// let route = Route::CreateMessage {
    ///     channel_id: 123,
    ///  };
    ///
    /// // Take a ticket from the ratelimiter.
    /// let rx = ratelimiter.get(route.path()).await;
    ///
    /// // Wait to be told that a request can be made...
    /// let _tx = rx.await;
    ///
    /// // The request can now be made.
    /// # Ok(()) }
    /// ```
    ///
    /// [`Ratelimiter`]: crate::ratelimiting::Ratelimiter
    #[allow(clippy::too_many_lines)]
    pub const fn path(&self) -> Path {
        match self {
            Self::AddGuildMember { guild_id, .. }
            | Self::GetMember { guild_id, .. }
            | Self::RemoveMember { guild_id, .. }
            | Self::UpdateMember { guild_id, .. } => Path::GuildsIdMembersId(*guild_id),
            Self::AddMemberRole { guild_id, .. } | Self::RemoveMemberRole { guild_id, .. } => {
                Path::GuildsIdMembersIdRolesId(*guild_id)
            }
            Self::CreateBan { guild_id, .. } | Self::DeleteBan { guild_id, .. } => {
                Path::GuildsIdBansUserId(*guild_id)
            }
            Self::CreateChannel { guild_id } => Path::GuildsIdChannels(*guild_id),
            Self::CreateEmoji { guild_id } | Self::GetEmojis { guild_id } => {
                Path::GuildsIdEmojis(*guild_id)
            }
            Self::CreateGlobalCommand { application_id }
            | Self::GetGlobalCommands { application_id }
            | Self::SetGlobalCommands { application_id } => {
                Path::ApplicationCommand(*application_id)
            }
            Self::CreateGuild | Self::CreateGuildFromTemplate { .. } | Self::GetTemplate { .. } => {
                Path::Guilds
            }
            Self::CreateGuildCommand { application_id, .. }
            | Self::DeleteGuildCommand { application_id, .. }
            | Self::GetGuildCommandPermissions { application_id, .. }
            | Self::GetGuildCommands { application_id, .. }
            | Self::SetCommandPermissions { application_id, .. }
            | Self::SetGuildCommands { application_id, .. }
            | Self::UpdateGuildCommand { application_id, .. } => {
                Path::ApplicationGuildCommand(*application_id)
            }
            Self::CreateGuildIntegration { guild_id } => Path::GuildsIdIntegrationsId(*guild_id),
            Self::CreateGuildPrune { guild_id, .. } | Self::GetGuildPruneCount { guild_id, .. } => {
                Path::GuildsIdPrune(*guild_id)
            }
            Self::CreateInvite { channel_id } | Self::GetChannelInvites { channel_id } => {
                Path::ChannelsIdInvites(*channel_id)
            }
            Self::CreateMessage { channel_id } | Self::GetMessages { channel_id, .. } => {
                Path::ChannelsIdMessages(*channel_id)
            }
            Self::CreatePrivateChannel | Self::GetUserPrivateChannels => Path::UsersIdChannels,
            Self::CreateReaction { channel_id, .. }
            | Self::DeleteReactionCurrentUser { channel_id, .. }
            | Self::DeleteReaction { channel_id, .. } => {
                Path::ChannelsIdMessagesIdReactionsUserIdType(*channel_id)
            }
            Self::CreateRole { guild_id } | Self::GetGuildRoles { guild_id } => {
                Path::GuildsIdRoles(*guild_id)
            }
            Self::CreateStageInstance { .. }
            | Self::DeleteStageInstance { .. }
            | Self::GetStageInstance { .. }
            | Self::UpdateStageInstance { .. } => Path::StageInstances,
            Self::CreateTemplate { guild_id } | Self::GetTemplates { guild_id } => {
                Path::GuildsIdTemplates(*guild_id)
            }
            Self::CreateTypingTrigger { channel_id } => Path::ChannelsIdTyping(*channel_id),
            Self::CreateWebhook { channel_id } | Self::GetChannelWebhooks { channel_id } => {
                Path::ChannelsIdWebhooks(*channel_id)
            }
            Self::CrosspostMessage { channel_id, .. } => {
                Path::ChannelsIdMessagesIdCrosspost(*channel_id)
            }
            Self::DeleteChannel { channel_id } => Path::ChannelsId(*channel_id),
            Self::DeleteEmoji { guild_id, .. } => Path::GuildsIdEmojisId(*guild_id),
            Self::DeleteGlobalCommand { application_id, .. }
            | Self::UpdateGlobalCommand { application_id, .. } => {
                Path::ApplicationCommandId(*application_id)
            }
            Self::DeleteGuild { guild_id } => Path::GuildsId(*guild_id),
            Self::DeleteGuildIntegration { guild_id, .. }
            | Self::UpdateGuildIntegration { guild_id, .. } => {
                Path::GuildsIdIntegrationsId(*guild_id)
            }
            Self::DeleteInteractionOriginal { application_id, .. }
            | Self::UpdateInteractionOriginal { application_id, .. } => {
                Path::WebhooksIdTokenMessagesId(*application_id)
            }
            Self::DeleteInvite { .. }
            | Self::GetInvite { .. }
            | Self::GetInviteWithExpiration { .. } => Path::InvitesCode,
            Self::DeleteMessageReactions { channel_id, .. }
            | Self::DeleteMessageSpecificReaction { channel_id, .. }
            | Self::GetReactionUsers { channel_id, .. } => {
                Path::ChannelsIdMessagesIdReactions(*channel_id)
            }
            Self::DeleteMessage { message_id, .. } => {
                Path::ChannelsIdMessagesId(Method::Delete, *message_id)
            }
            Self::DeleteMessages { channel_id } => Path::ChannelsIdMessagesBulkDelete(*channel_id),
            Self::DeletePermissionOverwrite { channel_id, .. }
            | Self::UpdatePermissionOverwrite { channel_id, .. } => {
                Path::ChannelsIdPermissionsOverwriteId(*channel_id)
            }
            Self::DeleteRole { guild_id, .. }
            | Self::UpdateRole { guild_id, .. }
            | Self::UpdateRolePositions { guild_id } => Path::GuildsIdRolesId(*guild_id),
            Self::DeleteTemplate { guild_id, .. }
            | Self::SyncTemplate { guild_id, .. }
            | Self::UpdateTemplate { guild_id, .. } => (Path::GuildsIdTemplatesCode(*guild_id)),
            Self::DeleteWebhookMessage { webhook_id, .. }
            | Self::GetWebhookMessage { webhook_id, .. }
            | Self::UpdateWebhookMessage { webhook_id, .. } => {
                Path::WebhooksIdTokenMessagesId(*webhook_id)
            }
            Self::DeleteWebhook { webhook_id, .. }
            | Self::ExecuteWebhook { webhook_id, .. }
            | Self::GetWebhook { webhook_id, .. }
            | Self::UpdateWebhook { webhook_id, .. } => (Path::WebhooksId(*webhook_id)),
            Self::FollowNewsChannel { channel_id } => Path::ChannelsIdFollowers(*channel_id),
            Self::GetAuditLogs { guild_id, .. } => Path::GuildsIdAuditLogs(*guild_id),
            Self::GetBan { guild_id, .. } => Path::GuildsIdBansId(*guild_id),
            Self::GetBans { guild_id } => Path::GuildsIdBans(*guild_id),
            Self::GetGatewayBot => Path::GatewayBot,
            Self::GetChannel { channel_id } | Self::UpdateChannel { channel_id } => {
                Path::ChannelsId(*channel_id)
            }
            Self::GetChannels { guild_id } | Self::UpdateGuildChannels { guild_id } => {
                Path::GuildsIdChannels(*guild_id)
            }
            Self::GetCommandPermissions { application_id, .. }
            | Self::UpdateCommandPermissions { application_id, .. } => {
                Path::ApplicationGuildCommandId(*application_id)
            }
            Self::GetCurrentUserApplicationInfo => Path::OauthApplicationsMe,
            Self::GetCurrentUser | Self::GetUser { .. } | Self::UpdateCurrentUser => Path::UsersId,
            Self::GetEmoji { guild_id, .. } | Self::UpdateEmoji { guild_id, .. } => {
                Path::GuildsIdEmojisId(*guild_id)
            }
            Self::GetGateway => Path::Gateway,
            Self::GetGuild { guild_id, .. } | Self::UpdateGuild { guild_id } => {
                Path::GuildsId(*guild_id)
            }
            Self::GetGuildWidget { guild_id } | Self::UpdateGuildWidget { guild_id } => {
                Path::GuildsIdWidget(*guild_id)
            }
            Self::GetGuildIntegrations { guild_id } => Path::GuildsIdIntegrations(*guild_id),
            Self::GetGuildInvites { guild_id } => Path::GuildsIdInvites(*guild_id),
            Self::GetGuildMembers { guild_id, .. } => Path::GuildsIdMembers(*guild_id),
            Self::GetGuildPreview { guild_id } => Path::GuildsIdPreview(*guild_id),
            Self::GetGuildVanityUrl { guild_id } => Path::GuildsIdVanityUrl(*guild_id),
            Self::GetGuildVoiceRegions { guild_id } => Path::GuildsIdRegions(*guild_id),
            Self::GetGuildWelcomeScreen { guild_id }
            | Self::UpdateGuildWelcomeScreen { guild_id } => Path::GuildsIdWelcomeScreen(*guild_id),
            Self::GetGuildWebhooks { guild_id } => Path::GuildsIdWebhooks(*guild_id),
            Self::GetGuilds { .. } => Path::UsersIdGuilds,
            Self::GetMessage { channel_id, .. } => {
                Path::ChannelsIdMessagesId(Method::Get, *channel_id)
            }
            Self::GetPins { channel_id } | Self::PinMessage { channel_id, .. } => {
                Path::ChannelsIdPins(*channel_id)
            }
            Self::GetUserConnections => Path::UsersIdConnections,
            Self::GetVoiceRegions => Path::VoiceRegions,
            Self::InteractionCallback { interaction_id, .. } => {
                Path::InteractionCallback(*interaction_id)
            }
            Self::LeaveGuild { .. } => Path::UsersIdGuildsId,
            Self::SearchGuildMembers { guild_id, .. } => Path::GuildsIdMembersSearch(*guild_id),
            Self::SyncGuildIntegration { guild_id, .. } => {
                Path::GuildsIdIntegrationsIdSync(*guild_id)
            }
            Self::UnpinMessage { channel_id, .. } => Path::ChannelsIdPinsMessageId(*channel_id),
            Self::UpdateCurrentUserVoiceState { guild_id }
            | Self::UpdateUserVoiceState { guild_id, .. } => Path::GuildsIdVoiceStates(*guild_id),
            Self::UpdateMessage { channel_id, .. } => {
                Path::ChannelsIdMessagesId(Method::Patch, *channel_id)
            }
            Self::UpdateNickname { guild_id } => Path::GuildsIdMembersMeNick(*guild_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Route;
    use crate::request::Method;

    /// Test a route for each method.
    #[test]
    fn test_methods() {
        assert_eq!(
            Method::Delete,
            Route::DeleteInvite {
                code: "twilight-rs",
            }
            .method()
        );
        assert_eq!(
            Method::Get,
            Route::GetInvite {
                code: "twilight-rs",
                with_counts: false,
            }
            .method()
        );
        assert_eq!(
            Method::Patch,
            Route::UpdateMessage {
                channel_id: 123,
                message_id: 456,
            }
            .method()
        );
        assert_eq!(Method::Post, Route::CreateGuild.method());
        assert_eq!(
            Method::Put,
            Route::SyncTemplate {
                guild_id: 123,
                template_code: "abc",
            }
            .method()
        );
    }
}
