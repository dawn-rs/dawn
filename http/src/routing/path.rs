use crate::request::Method;
use std::{
    convert::TryFrom,
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug)]
pub struct PathParseError {
    kind: PathParseErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl PathParseError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &PathParseErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (PathParseErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }
}

impl Display for PathParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            PathParseErrorType::IntegerParsing { .. } => {
                f.write_str("An ID in a segment was invalid")
            }
            PathParseErrorType::MessageIdWithoutMethod { .. } => {
                f.write_str("A message path was detected but the method wasn't given")
            }
            PathParseErrorType::NoMatch => f.write_str("There was no matched path"),
        }
    }
}

impl Error for PathParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PathParseErrorType {
    /// The ID couldn't be parsed as an integer.
    IntegerParsing,
    /// When parsing into a [`Path::ChannelsIdMessagesId`] variant, the method
    /// must also be specified via its `TryFrom` impl.
    MessageIdWithoutMethod {
        /// The ID of the channel.
        channel_id: u64,
    },
    /// A static path for the provided path string wasn't found.
    NoMatch,
}

/// An enum representing a path, most useful for ratelimiting implementations.
// If adding to this enum, be sure to add to the `TryFrom` impl.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Path {
    /// Operating on global commands.
    ApplicationCommand(u64),
    /// Operating on a specific command.
    ApplicationCommandId(u64),
    /// Operating on commands in a guild.
    ApplicationGuildCommand(u64),
    /// Operating on a specific command in a guild.
    ApplicationGuildCommandId(u64),
    /// Operating on a channel.
    ChannelsId(u64),
    /// Operating on a channel's invites.
    ChannelsIdInvites(u64),
    /// Operating on a channel's messages.
    ChannelsIdMessages(u64),
    /// Operating on a channel's messages by bulk deleting.
    ChannelsIdMessagesBulkDelete(u64),
    /// Operating on an individual channel's message.
    ChannelsIdMessagesId(Method, u64),
    /// Crossposting an individual channel's message.
    ChannelsIdMessagesIdCrosspost(u64),
    /// Operating on an individual channel's message's reactions.
    ChannelsIdMessagesIdReactions(u64),
    /// Operating on an individual channel's message's reactions while
    /// specifying the user ID and emoji type.
    ChannelsIdMessagesIdReactionsUserIdType(u64),
    /// Operating on an individual channel's message's threads.
    ChannelsIdMessagesIdThreads(u64),
    /// Operating on a channel's permission overwrites by ID.
    ChannelsIdPermissionsOverwriteId(u64),
    /// Operating on a channel's pins.
    ChannelsIdPins(u64),
    /// Operating on a channel's individual pinned message.
    ChannelsIdPinsMessageId(u64),
    /// Operating on a group DM's recipients.
    ChannelsIdRecipients(u64),
    /// Operating on a thread's members.
    ChannelsIdThreadMembers(u64),
    /// Operating on a channel's threads.
    ChannelsIdThreads(u64),
    /// Operating on a channel's typing indicator.
    ChannelsIdTyping(u64),
    /// Operating on a channel's webhooks.
    ChannelsIdWebhooks(u64),
    /// Operating on a channel's followers.
    ChannelsIdFollowers(u64),
    /// Operating with the gateway information.
    Gateway,
    /// Operating with the gateway information tailored to the current user.
    GatewayBot,
    /// Operating on the guild resource.
    Guilds,
    /// Operating on one of user's guilds.
    GuildsId(u64),
    GuildsIdBans(u64),
    GuildsIdBansId(u64),
    GuildsIdAuditLogs(u64),
    GuildsIdBansUserId(u64),
    GuildsIdChannels(u64),
    GuildsIdWidget(u64),
    GuildsIdEmojis(u64),
    GuildsIdEmojisId(u64),
    GuildsIdIntegrations(u64),
    GuildsIdIntegrationsId(u64),
    GuildsIdIntegrationsIdSync(u64),
    GuildsIdInvites(u64),
    GuildsIdMembers(u64),
    GuildsIdMembersId(u64),
    GuildsIdMembersIdRolesId(u64),
    GuildsIdMembersMeNick(u64),
    GuildsIdMembersSearch(u64),
    GuildsIdPreview(u64),
    GuildsIdPrune(u64),
    GuildsIdRegions(u64),
    GuildsIdRoles(u64),
    GuildsIdRolesId(u64),
    GuildsIdTemplates(u64),
    GuildsIdTemplatesCode(u64),
    GuildsIdVanityUrl(u64),
    GuildsIdVoiceStates(u64),
    GuildsIdWelcomeScreen(u64),
    GuildsIdWebhooks(u64),
    InvitesCode,
    /// Operating on an interaction's callback.
    InteractionCallback(u64),
    StageInstances,
    UsersId,
    OauthApplicationsMe,
    UsersIdConnections,
    UsersIdChannels,
    /// Operating on the state of a guild that the user is in.
    UsersIdGuilds,
    /// Operating on the state of a guild that the user is in.
    UsersIdGuildsId,
    /// Operating on the voice regions available to the current user.
    VoiceRegions,
    /// Operating on a message created by a webhook.
    WebhooksIdTokenMessagesId(u64),
    /// Operating on a webhook.
    WebhooksId(u64),
}

impl FromStr for Path {
    type Err = PathParseError;

    /// Parses a string into a path.
    ///
    /// The string *may* start with a slash (`/`), which will be ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use twilight_http::routing::Path;
    /// use std::str::FromStr;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert_eq!(Path::VoiceRegions, Path::from_str("/voice/regions")?);
    /// assert_eq!(
    ///     Path::ChannelsIdMessages(123),
    ///     Path::from_str("channels/123/messages")?,
    /// );
    /// # Ok(()) }
    /// ```
    #[allow(clippy::enum_glob_use, clippy::too_many_lines)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Path::*;

        fn parse_id(id: &str) -> Result<u64, PathParseError> {
            id.parse().map_err(|source| PathParseError {
                kind: PathParseErrorType::IntegerParsing,
                source: Some(Box::new(source)),
            })
        }

        let skip = usize::from(s.starts_with('/'));

        let parts = s.split('/').skip(skip).collect::<Vec<&str>>();

        #[allow(clippy::unnested_or_patterns)]
        Ok(match parts.as_slice() {
            ["applications", id, "commands"] => ApplicationCommand(parse_id(id)?),
            ["applications", id, "commands", _] => ApplicationCommandId(parse_id(id)?),
            ["applications", id, "guilds", _, "commands"]
            | ["applications", id, "guilds", _, "commands", "permissions"] => {
                ApplicationGuildCommand(parse_id(id)?)
            }
            ["applications", id, "guilds", _, "commands", _]
            | ["applications", id, "guilds", _, "commands", _, "permissions"] => {
                ApplicationGuildCommandId(parse_id(id)?)
            }
            ["channels", id] => ChannelsId(parse_id(id)?),
            ["channels", id, "followers"] => ChannelsIdFollowers(parse_id(id)?),
            ["channels", id, "invites"] => ChannelsIdInvites(parse_id(id)?),
            ["channels", id, "messages"] => ChannelsIdMessages(parse_id(id)?),
            ["channels", id, "messages", "bulk-delete"] => {
                ChannelsIdMessagesBulkDelete(parse_id(id)?)
            }
            ["channels", id, "messages", _] => {
                // can not map to path without method since they have different ratelimits
                return Err(PathParseError {
                    kind: PathParseErrorType::MessageIdWithoutMethod {
                        channel_id: parse_id(id)?,
                    },
                    source: None,
                });
            }
            ["channels", id, "messages", _, "crosspost"] => {
                ChannelsIdMessagesIdCrosspost(parse_id(id)?)
            }
            ["channels", id, "messages", _, "reactions"]
            | ["channels", id, "messages", _, "reactions", _] => {
                ChannelsIdMessagesIdReactions(parse_id(id)?)
            }
            ["channels", id, "messages", _, "reactions", _, _] => {
                ChannelsIdMessagesIdReactionsUserIdType(parse_id(id)?)
            }
            ["channels", id, "messages", _, "threads"] => {
                ChannelsIdMessagesIdThreads(parse_id(id)?)
            }
            ["channels", id, "permissions", _] => ChannelsIdPermissionsOverwriteId(parse_id(id)?),
            ["channels", id, "pins"] => ChannelsIdPins(parse_id(id)?),
            ["channels", id, "pins", _] => ChannelsIdPinsMessageId(parse_id(id)?),
            ["channels", id, "recipients"] | ["channels", id, "recipients", _] => {
                ChannelsIdRecipients(parse_id(id)?)
            }
            ["channels", id, "thread-members"] => ChannelsIdThreadMembers(parse_id(id)?),
            ["channels", id, "threads"] => ChannelsIdThreads(parse_id(id)?),
            ["channels", id, "typing"] => ChannelsIdTyping(parse_id(id)?),
            ["channels", id, "webhooks"] | ["channels", id, "webhooks", _] => {
                ChannelsIdWebhooks(parse_id(id)?)
            }
            ["gateway"] => Gateway,
            ["gateway", "bot"] => GatewayBot,
            ["guilds"] => Guilds,
            ["guilds", id] => GuildsId(parse_id(id)?),
            ["guilds", id, "audit-logs"] => GuildsIdAuditLogs(parse_id(id)?),
            ["guilds", id, "bans"] => GuildsIdBans(parse_id(id)?),
            ["guilds", id, "bans", _] => GuildsIdBansUserId(parse_id(id)?),
            ["guilds", id, "channels"] => GuildsIdChannels(parse_id(id)?),
            ["guilds", id, "widget"] | ["guilds", id, "widget.json"] => {
                GuildsIdWidget(parse_id(id)?)
            }
            ["guilds", id, "emojis"] => GuildsIdEmojis(parse_id(id)?),
            ["guilds", id, "emojis", _] => GuildsIdEmojisId(parse_id(id)?),
            ["guilds", id, "integrations"] => GuildsIdIntegrations(parse_id(id)?),
            ["guilds", id, "integrations", _] => GuildsIdIntegrationsId(parse_id(id)?),
            ["guilds", id, "integrations", _, "sync"] => GuildsIdIntegrationsIdSync(parse_id(id)?),
            ["guilds", id, "invites"] => GuildsIdInvites(parse_id(id)?),
            ["guilds", id, "members"] => GuildsIdMembers(parse_id(id)?),
            ["guilds", id, "members", "search"] => GuildsIdMembersSearch(parse_id(id)?),
            ["guilds", id, "members", _] => GuildsIdMembersId(parse_id(id)?),
            ["guilds", id, "members", _, "roles", _] => GuildsIdMembersIdRolesId(parse_id(id)?),
            ["guilds", id, "members", "@me", "nick"] => GuildsIdMembersMeNick(parse_id(id)?),
            ["guilds", id, "preview"] => GuildsIdPreview(parse_id(id)?),
            ["guilds", id, "prune"] => GuildsIdPrune(parse_id(id)?),
            ["guilds", id, "regions"] => GuildsIdRegions(parse_id(id)?),
            ["guilds", id, "roles"] => GuildsIdRoles(parse_id(id)?),
            ["guilds", id, "roles", _] => GuildsIdRolesId(parse_id(id)?),
            ["guilds", id, "templates"] => GuildsIdTemplates(parse_id(id)?),
            ["guilds", id, "templates", _] => GuildsIdTemplatesCode(parse_id(id)?),
            ["guilds", id, "vanity-url"] => GuildsIdVanityUrl(parse_id(id)?),
            ["guilds", id, "voice-states", _] => GuildsIdVoiceStates(parse_id(id)?),
            ["guilds", id, "welcome-screen"] => GuildsIdWelcomeScreen(parse_id(id)?),
            ["guilds", id, "webhooks"] => GuildsIdWebhooks(parse_id(id)?),
            ["invites", _] => InvitesCode,
            ["interactions", id, _, "callback"] => InteractionCallback(parse_id(id)?),
            ["stage-instances", _] => StageInstances,
            ["oauth2", "applications", "@me"] => OauthApplicationsMe,
            ["users", _] => UsersId,
            ["users", _, "connections"] => UsersIdConnections,
            ["users", _, "channels"] => UsersIdChannels,
            ["users", _, "guilds"] => UsersIdGuilds,
            ["users", _, "guilds", _] => UsersIdGuildsId,
            ["voice", "regions"] => VoiceRegions,
            ["webhooks", id] | ["webhooks", id, _] => WebhooksId(parse_id(id)?),
            ["webhooks", id, _, "messages", _] => WebhooksIdTokenMessagesId(parse_id(id)?),
            _ => {
                return Err(PathParseError {
                    kind: PathParseErrorType::NoMatch,
                    source: None,
                })
            }
        })
    }
}

impl TryFrom<(Method, &str)> for Path {
    type Error = PathParseError;

    fn try_from((method, s): (Method, &str)) -> Result<Self, Self::Error> {
        match Self::from_str(s) {
            Ok(v) => Ok(v),
            Err(why) => {
                if let PathParseErrorType::MessageIdWithoutMethod { channel_id } = why.kind() {
                    Ok(Self::ChannelsIdMessagesId(method, *channel_id))
                } else {
                    Err(why)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Path, PathParseError, PathParseErrorType};
    use crate::request::Method;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{convert::TryFrom, error::Error, fmt::Debug, hash::Hash, str::FromStr};

    assert_fields!(PathParseErrorType::MessageIdWithoutMethod: channel_id);
    assert_impl_all!(PathParseErrorType: Debug, Send, Sync);
    assert_impl_all!(PathParseError: Error, Send, Sync);
    assert_impl_all!(Path: Clone, Debug, Eq, Hash, PartialEq, Send, Sync);

    #[test]
    fn test_prefix_unimportant() -> Result<(), Box<dyn Error>> {
        assert_eq!(Path::Guilds, Path::from_str("guilds")?);
        assert_eq!(Path::Guilds, Path::from_str("/guilds")?);

        Ok(())
    }

    #[test]
    fn test_from_str() -> Result<(), Box<dyn Error>> {
        assert_eq!(Path::ChannelsId(123), Path::from_str("/channels/123")?);
        assert_eq!(Path::WebhooksId(123), Path::from_str("/webhooks/123")?);
        assert_eq!(Path::InvitesCode, Path::from_str("/invites/abc")?);

        Ok(())
    }

    #[test]
    fn test_message_id() -> Result<(), Box<dyn Error>> {
        assert!(matches!(
            Path::from_str("channels/123/messages/456")
                .unwrap_err()
                .kind(),
            PathParseErrorType::MessageIdWithoutMethod { channel_id: 123 },
        ));
        assert_eq!(
            Path::ChannelsIdMessagesId(Method::Get, 123),
            Path::try_from((Method::Get, "/channels/123/messages/456"))?,
        );

        Ok(())
    }
}
