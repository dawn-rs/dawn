//! Used when recieving interactions through gateway or webhooks.

pub mod application_command;
pub mod message_component;

mod interaction_type;
mod ping;

pub use self::{
    application_command::ApplicationCommand, interaction_type::InteractionType,
    message_component::MessageComponentInteraction, ping::Ping,
};

use crate::{
    channel::Message,
    guild::PartialMember,
    id::{ApplicationId, ChannelId, GuildId, InteractionId},
    user::User,
};
use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_value::Value;
use std::fmt::{Formatter, Result as FmtResult};

/// Payload received when a user executes an interaction.
///
/// Each variant corresponds to `InteractionType` in the discord docs. Refer to
/// [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#interaction
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Interaction {
    /// Ping variant.
    Ping(Box<Ping>),
    /// Application command variant.
    ApplicationCommand(Box<ApplicationCommand>),
    /// Message component variant.
    MessageComponent(Box<MessageComponentInteraction>),
}

impl Interaction {
    pub const fn guild_id(&self) -> Option<GuildId> {
        match self {
            Self::Ping(_) => None,
            Self::ApplicationCommand(inner) => inner.guild_id,
            Self::MessageComponent(inner) => inner.guild_id,
        }
    }
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(InteractionVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum InteractionField {
    ApplicationId,
    ChannelId,
    Data,
    GuildId,
    Id,
    Member,
    Message,
    Token,
    Type,
    User,
}

struct InteractionVisitor;

impl<'de> Visitor<'de> for InteractionVisitor {
    type Value = Interaction;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("enum Interaction")
    }

    #[allow(clippy::too_many_lines)]
    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        let mut application_id: Option<ApplicationId> = None;
        let mut channel_id: Option<ChannelId> = None;
        let mut data: Option<Value> = None;
        let mut guild_id: Option<Option<GuildId>> = None;
        let mut id: Option<InteractionId> = None;
        let mut member: Option<Option<PartialMember>> = None;
        let mut message: Option<Message> = None;
        let mut token: Option<String> = None;
        let mut kind: Option<InteractionType> = None;
        let mut user: Option<Option<User>> = None;

        let span = tracing::trace_span!("deserializing interaction");
        let _span_enter = span.enter();

        loop {
            let span_child = tracing::trace_span!("iterating over interaction");
            let _span_child_enter = span_child.enter();

            let key = match map.next_key() {
                Ok(Some(key)) => {
                    tracing::trace!(?key, "found key");

                    key
                }
                Ok(None) => break,
                Err(why) => {
                    // Encountered when we run into an unknown key.
                    map.next_value::<IgnoredAny>()?;

                    tracing::trace!("ran into an unknown key: {:?}", why);

                    continue;
                }
            };

            match key {
                InteractionField::ApplicationId => {
                    if application_id.is_some() {
                        return Err(DeError::duplicate_field("application_id"));
                    }

                    application_id = Some(map.next_value()?);
                }
                InteractionField::ChannelId => {
                    if channel_id.is_some() {
                        return Err(DeError::duplicate_field("channel_id"));
                    }

                    channel_id = Some(map.next_value()?);
                }
                InteractionField::Data => {
                    if data.is_some() {
                        return Err(DeError::duplicate_field("data"));
                    }

                    data = Some(map.next_value()?);
                }
                InteractionField::GuildId => {
                    if guild_id.is_some() {
                        return Err(DeError::duplicate_field("guild_id"));
                    }

                    guild_id = Some(map.next_value()?);
                }
                InteractionField::Id => {
                    if id.is_some() {
                        return Err(DeError::duplicate_field("id"));
                    }

                    id = Some(map.next_value()?);
                }
                InteractionField::Member => {
                    if member.is_some() {
                        return Err(DeError::duplicate_field("member"));
                    }

                    member = Some(map.next_value()?);
                }
                InteractionField::Message => {
                    if message.is_some() {
                        return Err(DeError::duplicate_field("message"));
                    }

                    message = Some(map.next_value()?);
                }
                InteractionField::Token => {
                    if token.is_some() {
                        return Err(DeError::duplicate_field("token"));
                    }

                    token = Some(map.next_value()?);
                }
                InteractionField::Type => {
                    if kind.is_some() {
                        return Err(DeError::duplicate_field("kind"));
                    }

                    kind = Some(map.next_value()?);
                }
                InteractionField::User => {
                    if user.is_some() {
                        return Err(DeError::duplicate_field("user"));
                    }

                    user = Some(map.next_value()?);
                }
            }
        }

        let application_id =
            application_id.ok_or_else(|| DeError::missing_field("application_id"))?;
        let id = id.ok_or_else(|| DeError::missing_field("id"))?;
        let token = token.ok_or_else(|| DeError::missing_field("token"))?;
        let kind = kind.ok_or_else(|| DeError::missing_field("kind"))?;

        tracing::trace!(
            %application_id,
            %id,
            %token,
            ?kind,
            "common fields of all variants exist"
        );

        Ok(match kind {
            InteractionType::Ping => {
                tracing::trace!("handling ping");

                Self::Value::Ping(Box::new(Ping {
                    application_id,
                    id,
                    kind,
                    token,
                }))
            }
            InteractionType::ApplicationCommand => {
                let channel_id = channel_id.ok_or_else(|| DeError::missing_field("channel_id"))?;
                let data = data
                    .ok_or_else(|| DeError::missing_field("data"))?
                    .deserialize_into()
                    .map_err(|_| DeError::custom("expected CommandData struct"))?;

                let guild_id = guild_id.unwrap_or_default();
                let member = member.unwrap_or_default();
                let user = user.unwrap_or_default();

                tracing::trace!(%channel_id, "handling application command");

                Self::Value::ApplicationCommand(Box::new(ApplicationCommand {
                    application_id,
                    channel_id,
                    data,
                    guild_id,
                    id,
                    kind,
                    member,
                    token,
                    user,
                }))
            }
            InteractionType::MessageComponent => {
                let channel_id = channel_id.ok_or_else(|| DeError::missing_field("channel_id"))?;
                let data = data
                    .ok_or_else(|| DeError::missing_field("data"))?
                    .deserialize_into()
                    .map_err(|_| {
                        DeError::custom("expected MessageComponentInteractionData struct")
                    })?;
                let message = message.ok_or_else(|| DeError::missing_field("message"))?;

                let guild_id = guild_id.unwrap_or_default();
                let member = member.unwrap_or_default();
                let user = user.unwrap_or_default();

                Self::Value::MessageComponent(Box::new(MessageComponentInteraction {
                    application_id,
                    channel_id,
                    data,
                    guild_id,
                    id,
                    kind,
                    member,
                    message,
                    token,
                    user,
                }))
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        application::interaction::{
            application_command::{
                ApplicationCommand, CommandData, CommandDataOption, CommandInteractionDataResolved,
            },
            Interaction, InteractionType,
        },
        guild::{PartialMember, Permissions},
        id::{ApplicationId, ChannelId, CommandId, GuildId, InteractionId, UserId},
        user::User,
    };
    use serde_test::Token;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_interaction_full() {
        let value = Interaction::ApplicationCommand(Box::new(ApplicationCommand {
            application_id: ApplicationId(100),
            channel_id: ChannelId(200),
            data: CommandData {
                id: CommandId(300),
                name: "command name".into(),
                options: vec![CommandDataOption::String {
                    name: "member".into(),
                    value: "600".into(),
                }],
                resolved: Some(CommandInteractionDataResolved {
                    channels: Vec::new(),
                    members: Vec::new(),
                    roles: Vec::new(),
                    users: vec![User {
                        avatar: Some("avatar string".into()),
                        bot: false,
                        discriminator: "1111".into(),
                        email: None,
                        flags: None,
                        id: UserId(600),
                        locale: None,
                        mfa_enabled: None,
                        name: "username".into(),
                        premium_type: None,
                        public_flags: None,
                        system: None,
                        verified: None,
                    }],
                }),
            },
            guild_id: Some(GuildId(400)),
            id: InteractionId(500),
            kind: InteractionType::ApplicationCommand,
            member: Some(PartialMember {
                deaf: false,
                joined_at: Some("joined at".into()),
                mute: false,
                nick: Some("nickname".into()),
                permissions: Some(Permissions::empty()),
                premium_since: None,
                roles: Vec::new(),
                user: Some(User {
                    avatar: Some("avatar string".into()),
                    bot: false,
                    discriminator: "1111".into(),
                    email: None,
                    flags: None,
                    id: UserId(600),
                    locale: None,
                    mfa_enabled: None,
                    name: "username".into(),
                    premium_type: None,
                    public_flags: None,
                    system: None,
                    verified: None,
                }),
            }),
            token: "interaction token".into(),
            user: None,
        }));

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Interaction",
                    len: 8,
                },
                Token::Str("application_id"),
                Token::NewtypeStruct {
                    name: "ApplicationId",
                },
                Token::Str("100"),
                Token::Str("channel_id"),
                Token::NewtypeStruct { name: "ChannelId" },
                Token::Str("200"),
                Token::Str("data"),
                Token::Struct {
                    name: "CommandData",
                    len: 4,
                },
                Token::Str("id"),
                Token::NewtypeStruct { name: "CommandId" },
                Token::Str("300"),
                Token::Str("name"),
                Token::Str("command name"),
                Token::Str("options"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "CommandDataOption",
                    len: 2,
                },
                Token::Str("name"),
                Token::Str("member"),
                Token::Str("value"),
                Token::Str("600"),
                Token::StructEnd,
                Token::SeqEnd,
                Token::Str("resolved"),
                Token::Some,
                Token::Struct {
                    name: "CommandInteractionDataResolved",
                    len: 1,
                },
                Token::Str("users"),
                Token::Map { len: Some(1) },
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("600"),
                Token::Struct {
                    name: "User",
                    len: 5,
                },
                Token::Str("avatar"),
                Token::Some,
                Token::Str("avatar string"),
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("1111"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("600"),
                Token::Str("username"),
                Token::Str("username"),
                Token::StructEnd,
                Token::MapEnd,
                Token::StructEnd,
                Token::StructEnd,
                Token::Str("guild_id"),
                Token::Some,
                Token::NewtypeStruct { name: "GuildId" },
                Token::Str("400"),
                Token::Str("id"),
                Token::NewtypeStruct {
                    name: "InteractionId",
                },
                Token::Str("500"),
                Token::Str("type"),
                Token::U8(2),
                Token::Str("member"),
                Token::Some,
                Token::Struct {
                    name: "PartialMember",
                    len: 7,
                },
                Token::Str("deaf"),
                Token::Bool(false),
                Token::Str("joined_at"),
                Token::Some,
                Token::Str("joined at"),
                Token::Str("mute"),
                Token::Bool(false),
                Token::Str("nick"),
                Token::Some,
                Token::Str("nickname"),
                Token::Str("permissions"),
                Token::Some,
                Token::Str("0"),
                Token::Str("roles"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("user"),
                Token::Some,
                Token::Struct {
                    name: "User",
                    len: 5,
                },
                Token::Str("avatar"),
                Token::Some,
                Token::Str("avatar string"),
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("1111"),
                Token::Str("id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("600"),
                Token::Str("username"),
                Token::Str("username"),
                Token::StructEnd,
                Token::StructEnd,
                Token::Str("token"),
                Token::Str("interaction token"),
                Token::StructEnd,
            ],
        );
    }
}
