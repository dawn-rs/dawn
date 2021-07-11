use super::route::Route;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RouteDisplay<'a>(&'a Route);

impl<'a> RouteDisplay<'a> {
    /// Create a display formatter for a route.
    ///
    /// This is equivalent to [`Route::display`].
    pub(super) const fn new(route: &'a Route) -> Self {
        Self(route)
    }

    /// Immutable reference to the underlying route.
    ///
    /// ```
    /// use twilight_http::routing::Route;
    ///
    /// let route = Route::GetMessage {
    ///     channel_id: 123,
    ///     message_id: 456,
    /// };
    /// let display = route.display();
    ///
    /// assert_eq!(display.route_ref(), &route);
    /// ```
    pub const fn route_ref(&self) -> &'a Route {
        self.0
    }
}

impl Display for RouteDisplay<'_> {
    // Notably, we don't use macros like `write!` or `format_args!` due to them
    // both compiling slowly and performing slowly during runtime.
    //
    // See:
    // <https://github.com/rust-lang/rust/issues/76490>
    // <https://github.com/rust-lang/rust/issues/10761>
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.0 {
            Route::AddGuildMember { guild_id, user_id }
            | Route::GetMember { guild_id, user_id }
            | Route::RemoveMember { guild_id, user_id }
            | Route::UpdateMember { guild_id, user_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/members/")?;

                Display::fmt(user_id, f)
            }
            Route::AddMemberRole {
                guild_id,
                role_id,
                user_id,
            }
            | Route::RemoveMemberRole {
                guild_id,
                role_id,
                user_id,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/members/")?;
                Display::fmt(user_id, f)?;
                f.write_str("/roles/")?;

                Display::fmt(role_id, f)
            }
            Route::AddThreadMember {
                channel_id,
                user_id,
            }
            | Route::RemoveThreadMember {
                channel_id,
                user_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/thread-members/")?;

                Display::fmt(user_id, f)
            }
            Route::CreateBan {
                guild_id,
                delete_message_days,
                reason,
                user_id,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/bans/")?;
                Display::fmt(user_id, f)?;
                f.write_str("?")?;

                if let Some(delete_message_days) = delete_message_days {
                    f.write_str("delete_message_days=")?;
                    Display::fmt(delete_message_days, f)?;

                    if reason.is_some() {
                        f.write_str("&")?;
                    }
                }

                if let Some(reason) = reason {
                    f.write_str("reason=")?;
                    let encoded_reason = utf8_percent_encode(&reason, NON_ALPHANUMERIC);

                    Display::fmt(&encoded_reason, f)?;
                }

                Ok(())
            }
            Route::CreateChannel { guild_id }
            | Route::GetChannels { guild_id }
            | Route::UpdateGuildChannels { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/channels")
            }
            Route::CreateEmoji { guild_id } | Route::GetEmojis { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/emojis")
            }
            Route::CreateGlobalCommand { application_id }
            | Route::GetGlobalCommands { application_id }
            | Route::SetGlobalCommands { application_id } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;

                f.write_str("/commands")
            }
            Route::CreateGuild => f.write_str("guilds"),
            Route::CreateGuildCommand {
                application_id,
                guild_id,
            }
            | Route::GetGuildCommands {
                application_id,
                guild_id,
            }
            | Route::SetGuildCommands {
                application_id,
                guild_id,
            } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/commands")
            }
            Route::CreateGuildFromTemplate { template_code }
            | Route::GetTemplate { template_code } => {
                f.write_str("guilds/templates/")?;

                f.write_str(template_code)
            }
            Route::CreateGuildIntegration { guild_id }
            | Route::GetGuildIntegrations { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/integrations")
            }
            Route::CreateGuildPrune {
                compute_prune_count,
                days,
                guild_id,
                include_roles,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/prune?")?;

                if let Some(compute_prune_count) = compute_prune_count {
                    f.write_str("compute_prune_count=")?;
                    Display::fmt(compute_prune_count, f)?;
                }

                if let Some(days) = days {
                    f.write_str("&days=")?;
                    Display::fmt(days, f)?;
                }

                if !include_roles.is_empty() {
                    let role_count = include_roles.len() - 1;

                    f.write_str("include_roles=")?;

                    for (idx, role_id) in include_roles.iter().enumerate() {
                        Display::fmt(role_id, f)?;

                        if idx < role_count {
                            f.write_str(",")?;
                        }
                    }
                }

                Ok(())
            }
            Route::CreateInvite { channel_id } | Route::GetChannelInvites { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/invites")
            }
            Route::CreateMessage { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/messages")
            }
            Route::CreatePrivateChannel | Route::GetUserPrivateChannels => {
                f.write_str("users/@me/channels")
            }
            Route::CreateReaction {
                channel_id,
                emoji,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;
                f.write_str("/reactions/")?;
                f.write_str(emoji)?;

                f.write_str("/@me")
            }
            Route::CreateRole { guild_id }
            | Route::GetGuildRoles { guild_id }
            | Route::UpdateRolePositions { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/roles")
            }
            Route::CreateStageInstance { .. } => f.write_str("stage-instances"),
            Route::CreateTemplate { guild_id } | Route::GetTemplates { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/templates")
            }
            Route::CreateThread { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/threads")
            }
            Route::CreateThreadFromMessage {
                channel_id,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;

                f.write_str("/threads")
            }
            Route::CreateTypingTrigger { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/typing")
            }
            Route::CreateWebhook { channel_id } | Route::GetChannelWebhooks { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/webhooks")
            }
            Route::CrosspostMessage {
                channel_id,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;

                f.write_str("/crosspost")
            }
            Route::DeleteBan { guild_id, user_id } | Route::GetBan { guild_id, user_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/bans/")?;

                Display::fmt(user_id, f)
            }
            Route::DeleteChannel { channel_id }
            | Route::GetChannel { channel_id }
            | Route::UpdateChannel { channel_id } => {
                f.write_str("channels/")?;

                Display::fmt(channel_id, f)
            }
            Route::DeleteEmoji { emoji_id, guild_id }
            | Route::GetEmoji { emoji_id, guild_id }
            | Route::UpdateEmoji { emoji_id, guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/emojis/")?;

                Display::fmt(emoji_id, f)
            }
            Route::DeleteGlobalCommand {
                application_id,
                command_id,
            }
            | Route::UpdateGlobalCommand {
                application_id,
                command_id,
            } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/commands/")?;

                Display::fmt(command_id, f)
            }
            Route::DeleteGuild { guild_id } | Route::UpdateGuild { guild_id } => {
                f.write_str("guilds/")?;

                Display::fmt(guild_id, f)
            }
            Route::DeleteGuildCommand {
                application_id,
                command_id,
                guild_id,
            }
            | Route::UpdateGuildCommand {
                application_id,
                command_id,
                guild_id,
            } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/commands/")?;

                Display::fmt(command_id, f)
            }
            Route::DeleteGuildIntegration {
                guild_id,
                integration_id,
            }
            | Route::UpdateGuildIntegration {
                guild_id,
                integration_id,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/integrations/")?;

                Display::fmt(integration_id, f)
            }
            Route::DeleteInteractionOriginal {
                application_id,
                interaction_token,
            }
            | Route::UpdateInteractionOriginal {
                application_id,
                interaction_token,
            } => {
                f.write_str("webhooks/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/")?;
                f.write_str(interaction_token)?;

                f.write_str("/messages/@original")
            }
            Route::DeleteInvite { code } => {
                f.write_str("invites/")?;

                f.write_str(code)
            }
            Route::DeleteMessageReactions {
                channel_id,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;

                f.write_str("/reactions")
            }
            Route::DeleteMessageSpecificReaction {
                channel_id,
                message_id,
                emoji,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;
                f.write_str("/reactions/")?;

                f.write_str(emoji)
            }
            Route::DeleteMessage {
                channel_id,
                message_id,
            }
            | Route::GetMessage {
                channel_id,
                message_id,
            }
            | Route::UpdateMessage {
                channel_id,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;

                Display::fmt(message_id, f)
            }
            Route::DeleteMessages { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/messages/bulk-delete")
            }
            Route::DeletePermissionOverwrite {
                channel_id,
                target_id,
            }
            | Route::UpdatePermissionOverwrite {
                channel_id,
                target_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/permissions/")?;

                Display::fmt(target_id, f)
            }
            Route::DeleteReaction {
                channel_id,
                emoji,
                message_id,
                user,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;
                f.write_str("/reactions/")?;
                f.write_str(emoji)?;
                f.write_str("/")?;

                f.write_str(user)
            }
            Route::DeleteRole { guild_id, role_id } | Route::UpdateRole { guild_id, role_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/roles/")?;

                Display::fmt(role_id, f)
            }
            Route::DeleteStageInstance { channel_id }
            | Route::GetStageInstance { channel_id }
            | Route::UpdateStageInstance { channel_id } => {
                f.write_str("stage-instances/")?;

                Display::fmt(channel_id, f)
            }
            Route::DeleteTemplate {
                guild_id,
                template_code,
            }
            | Route::SyncTemplate {
                guild_id,
                template_code,
            }
            | Route::UpdateTemplate {
                guild_id,
                template_code,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/templates/")?;

                f.write_str(template_code)
            }
            Route::DeleteWebhookMessage {
                message_id,
                token,
                webhook_id,
            }
            | Route::GetWebhookMessage {
                message_id,
                token,
                webhook_id,
            }
            | Route::UpdateWebhookMessage {
                message_id,
                token,
                webhook_id,
            } => {
                f.write_str("webhooks/")?;
                Display::fmt(webhook_id, f)?;
                f.write_str("/")?;
                f.write_str(token)?;
                f.write_str("/messages/")?;

                Display::fmt(message_id, f)
            }
            Route::DeleteWebhook { token, webhook_id }
            | Route::GetWebhook { token, webhook_id }
            | Route::UpdateWebhook { token, webhook_id } => {
                f.write_str("webhooks/")?;
                Display::fmt(webhook_id, f)?;

                if let Some(token) = token {
                    f.write_str("/")?;
                    f.write_str(token)?;
                }

                Ok(())
            }
            Route::ExecuteWebhook {
                token,
                wait,
                webhook_id,
            } => {
                f.write_str("webhooks/")?;
                Display::fmt(webhook_id, f)?;
                f.write_str("/")?;
                f.write_str(token)?;

                if let Some(wait) = wait {
                    f.write_str("?wait=")?;
                    f.write_str(if *wait { "true" } else { "false" })?;
                }

                Ok(())
            }
            Route::FollowNewsChannel { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/followers")
            }
            Route::GetActiveThreads { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/threads/active")
            }
            Route::GetAuditLogs {
                action_type,
                before,
                guild_id,
                limit,
                user_id,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/audit-logs?")?;

                if let Some(action_type) = action_type {
                    f.write_str("action_type=")?;
                    Display::fmt(action_type, f)?;
                }

                if let Some(before) = before {
                    f.write_str("&before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                if let Some(user_id) = user_id {
                    f.write_str("&user_id=")?;
                    Display::fmt(user_id, f)?;
                }

                Ok(())
            }
            Route::GetBans { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/bans")
            }
            Route::GetGatewayBot => f.write_str("gateway/bot"),
            Route::GetCommandPermissions {
                application_id,
                command_id,
                guild_id,
            }
            | Route::UpdateCommandPermissions {
                application_id,
                command_id,
                guild_id,
            } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/commands/")?;
                Display::fmt(command_id, f)?;

                f.write_str("/permissions")
            }
            Route::GetCurrentUserApplicationInfo => f.write_str("/oauth2/applications/@me"),
            Route::UpdateCurrentUser => f.write_str("users/@me"),
            Route::GetGateway => f.write_str("gateway"),
            Route::GetGuild {
                guild_id,
                with_counts,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                if *with_counts {
                    f.write_str("?with_counts=true")?;
                }

                Ok(())
            }
            Route::GetGuildCommandPermissions {
                application_id,
                guild_id,
            }
            | Route::SetCommandPermissions {
                application_id,
                guild_id,
            } => {
                f.write_str("applications/")?;
                Display::fmt(application_id, f)?;
                f.write_str("/guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/commands/permissions")
            }
            Route::GetGuildInvites { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/invites")
            }
            Route::GetGuildMembers {
                after,
                guild_id,
                limit,
                presences,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("members?")?;

                if let Some(after) = after {
                    f.write_str("after=")?;
                    Display::fmt(after, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                if let Some(presences) = presences {
                    f.write_str("&presences=")?;
                    Display::fmt(presences, f)?;
                }

                Ok(())
            }
            Route::GetGuildPreview { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/preview")
            }
            Route::GetGuildPruneCount {
                days,
                guild_id,
                include_roles,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/prune?")?;

                if let Some(days) = days {
                    f.write_str("days=")?;
                    Display::fmt(days, f)?;
                }

                if !include_roles.is_empty() {
                    f.write_str("&include_roles=")?;

                    let role_count = include_roles.len() - 1;

                    for (idx, role_id) in include_roles.iter().enumerate() {
                        Display::fmt(role_id, f)?;

                        if idx < role_count {
                            f.write_str(",")?;
                        }
                    }
                }

                Ok(())
            }
            Route::GetGuildVanityUrl { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/vanity-url")
            }
            Route::GetGuildVoiceRegions { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/regions")
            }
            Route::GetGuildWelcomeScreen { guild_id }
            | Route::UpdateGuildWelcomeScreen { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/welcome-screen")
            }
            Route::GetGuildWebhooks { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/webhooks")
            }
            Route::GetGuildWidget { guild_id } | Route::UpdateGuildWidget { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/widget")
            }
            Route::GetGuilds {
                after,
                before,
                limit,
            } => {
                f.write_str("users/@me/guilds?")?;

                if let Some(after) = after {
                    f.write_str("after=")?;
                    Display::fmt(after, f)?;
                }

                if let Some(before) = before {
                    f.write_str("&before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetInvite { code, with_counts } => {
                f.write_str("invites/")?;
                f.write_str(code)?;

                if *with_counts {
                    f.write_str("?with-counts=true")?;
                }

                Ok(())
            }
            Route::GetInviteWithExpiration {
                code,
                with_counts,
                with_expiration,
            } => {
                f.write_str("invites/")?;
                f.write_str(code)?;
                f.write_str("?")?;

                if *with_counts {
                    f.write_str("with-counts=true")?;
                }

                if *with_expiration {
                    f.write_str("with-expiration=true")?;
                }

                Ok(())
            }
            Route::GetMessages {
                channel_id,
                after,
                around,
                before,
                limit,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages?")?;

                if let Some(after) = after {
                    f.write_str("after=")?;
                    Display::fmt(after, f)?;
                }

                if let Some(around) = around {
                    f.write_str("&around=")?;
                    Display::fmt(around, f)?;
                }

                if let Some(before) = before {
                    f.write_str("&before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetPins { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/pins")
            }
            Route::GetJoinedPrivateArchivedThreads {
                before,
                channel_id,
                limit,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/users/@me/threads/archived/private?")?;

                if let Some(before) = before {
                    f.write_str("before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetPrivateArchivedThreads {
                before,
                channel_id,
                limit,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/threads/archived/private?")?;

                if let Some(before) = before {
                    f.write_str("before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetPublicArchivedThreads {
                before,
                channel_id,
                limit,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/threads/archived/public")?;

                if let Some(before) = before {
                    f.write_str("before=")?;
                    Display::fmt(before, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetReactionUsers {
                after,
                channel_id,
                emoji,
                limit,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/messages/")?;
                Display::fmt(message_id, f)?;
                f.write_str("/reactions/")?;
                f.write_str(emoji)?;
                f.write_str("?")?;

                if let Some(after) = after {
                    f.write_str("after=")?;
                    Display::fmt(after, f)?;
                }

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::GetThreadMembers { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/thread-members")
            }
            Route::GetUserConnections => f.write_str("users/@me/connections"),
            Route::GetUser { target_user } => {
                f.write_str("users/")?;

                f.write_str(target_user)
            }
            Route::GetVoiceRegions => f.write_str("voice/regions"),
            Route::InteractionCallback {
                interaction_id,
                interaction_token,
            } => {
                f.write_str("interactions/")?;
                Display::fmt(interaction_id, f)?;
                f.write_str("/")?;
                f.write_str(interaction_token)?;

                f.write_str("/callback")
            }
            Route::JoinThread { channel_id } | Route::LeaveThread { channel_id } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;

                f.write_str("/thread-members/@me")
            }
            Route::LeaveGuild { guild_id } => {
                f.write_str("users/@me/guilds/")?;

                Display::fmt(guild_id, f)
            }
            Route::PinMessage {
                channel_id,
                message_id,
            }
            | Route::UnpinMessage {
                channel_id,
                message_id,
            } => {
                f.write_str("channels/")?;
                Display::fmt(channel_id, f)?;
                f.write_str("/pins/")?;

                Display::fmt(message_id, f)
            }
            Route::SearchGuildMembers {
                guild_id,
                limit,
                query,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/members/search?query=")?;
                f.write_str(query)?;

                if let Some(limit) = limit {
                    f.write_str("&limit=")?;
                    Display::fmt(limit, f)?;
                }

                Ok(())
            }
            Route::SyncGuildIntegration {
                guild_id,
                integration_id,
            } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/integrations/")?;
                Display::fmt(integration_id, f)?;

                f.write_str("sync")
            }
            Route::UpdateCurrentUserVoiceState { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/voice-states/@me")
            }
            Route::UpdateNickname { guild_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;

                f.write_str("/members/@me/nick")
            }
            Route::UpdateUserVoiceState { guild_id, user_id } => {
                f.write_str("guilds/")?;
                Display::fmt(guild_id, f)?;
                f.write_str("/voices-states/")?;

                Display::fmt(user_id, f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Route, RouteDisplay};
    use static_assertions::assert_impl_all;
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
    };

    assert_impl_all!(RouteDisplay<'_>: Clone, Debug, Display, Eq, Hash, PartialEq, Send, Sync);

    #[test]
    fn test_set_guild_commands() {
        let route = Route::SetGuildCommands {
            application_id: 1,
            guild_id: 2,
        };

        assert_eq!(
            "applications/1/guilds/2/commands",
            route.display().to_string()
        );
    }

    #[test]
    fn test_update_global_command() {
        let route = Route::UpdateGlobalCommand {
            application_id: 1,
            command_id: 2,
        };

        assert_eq!("applications/1/commands/2", route.display().to_string());
    }
}
