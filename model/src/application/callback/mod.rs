//! Used when responding to interactions.

mod callback_data;
mod response_type;

pub use self::{callback_data::CallbackData, response_type::ResponseType};

use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::fmt::{Formatter, Result as FmtResult};

/// Payload used for responding to an interaction.
///
/// Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#interaction-response
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InteractionResponse {
    /// Used when responding to an interaction of type Ping.
    Pong,
    /// Responds to an interaction with a message.
    ChannelMessageWithSource(CallbackData),
    /// Acknowledges an interaction, showing a loading state.
    DeferredChannelMessageWithSource(CallbackData),
    /// Acknowledge an interaction and edit the original message later.
    ///
    /// This is only valid for components.
    DeferredUpdateMessage,
    /// Edit the message a component is attached to.
    UpdateMessage(CallbackData),
}

impl InteractionResponse {
    /// Type of response this is.
    ///
    /// # Examples
    ///
    /// Check the types of the [`DeferredUpdateMessage`] and [`Pong`]
    /// interaction response variants.
    ///
    /// ```
    /// use twilight_model::application::callback::{
    ///     InteractionResponse,
    ///     ResponseType,
    /// };
    ///
    /// assert_eq!(
    ///     ResponseType::DeferredUpdateMessage,
    ///     InteractionResponse::DeferredUpdateMessage.kind(),
    /// );
    /// assert_eq!(ResponseType::Pong, InteractionResponse::Pong.kind());
    /// ```
    ///
    /// [`DeferredUpdateMessage`]: Self::DeferredUpdateMessage
    /// [`Pong`]: Self::Pong
    pub const fn kind(&self) -> ResponseType {
        match self {
            Self::Pong => ResponseType::Pong,
            Self::ChannelMessageWithSource(_) => ResponseType::ChannelMessageWithSource,
            Self::DeferredChannelMessageWithSource(_) => {
                ResponseType::DeferredChannelMessageWithSource
            }
            Self::DeferredUpdateMessage => ResponseType::DeferredUpdateMessage,
            Self::UpdateMessage(_) => ResponseType::UpdateMessage,
        }
    }
}

impl<'de> Deserialize<'de> for InteractionResponse {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ResponseVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum ResponseField {
    Data,
    Type,
}

struct ResponseVisitor;

impl<'de> Visitor<'de> for ResponseVisitor {
    type Value = InteractionResponse;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("struct InteractionResponse")
    }

    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        let mut data: Option<CallbackData> = None;
        let mut kind: Option<ResponseType> = None;

        let span = tracing::trace_span!("deserializing interaction response");
        let _span_enter = span.enter();

        loop {
            let span_child = tracing::trace_span!("iterating over interaction response");
            let _span_child_enter = span_child.enter();

            let key = match map.next_key() {
                Ok(Some(key)) => {
                    tracing::trace!(?key, "found key");

                    key
                }
                Ok(None) => break,
                Err(why) => {
                    map.next_value::<IgnoredAny>()?;

                    tracing::trace!("ran into an unknown key: {:?}", why);

                    continue;
                }
            };

            match key {
                ResponseField::Data => {
                    if data.is_some() {
                        return Err(DeError::duplicate_field("data"));
                    }

                    data = Some(map.next_value()?);
                }
                ResponseField::Type => {
                    if kind.is_some() {
                        return Err(DeError::duplicate_field("type"));
                    }

                    kind = Some(map.next_value()?);
                }
            }
        }

        let kind = kind.ok_or_else(|| DeError::missing_field("type"))?;

        Ok(match kind {
            ResponseType::Pong => Self::Value::Pong,
            ResponseType::ChannelMessageWithSource => {
                let data = data.ok_or_else(|| DeError::missing_field("data"))?;

                Self::Value::ChannelMessageWithSource(data)
            }
            ResponseType::DeferredChannelMessageWithSource => {
                let data = data.ok_or_else(|| DeError::missing_field("data"))?;

                Self::Value::DeferredChannelMessageWithSource(data)
            }
            ResponseType::DeferredUpdateMessage => Self::Value::DeferredUpdateMessage,
            ResponseType::UpdateMessage => {
                let data = data.ok_or_else(|| DeError::missing_field("data"))?;

                Self::Value::UpdateMessage(data)
            }
        })
    }
}

impl Serialize for InteractionResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Pong | Self::DeferredUpdateMessage => {
                let mut state = serializer.serialize_struct("InteractionResponse", 1)?;

                state.serialize_field("type", &self.kind())?;

                state.end()
            }
            Self::ChannelMessageWithSource(data)
            | Self::DeferredChannelMessageWithSource(data)
            | Self::UpdateMessage(data) => {
                let mut state = serializer.serialize_struct("InteractionResponse", 2)?;

                state.serialize_field("type", &self.kind())?;
                state.serialize_field("data", &data)?;

                state.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CallbackData, InteractionResponse};
    use crate::channel::message::MessageFlags;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::assert_impl_all;
    use std::{fmt::Debug, hash::Hash};

    assert_impl_all!(
        InteractionResponse: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Send,
        Serialize,
        Sync
    );

    #[test]
    fn test_response() {
        let value = InteractionResponse::ChannelMessageWithSource(CallbackData {
            allowed_mentions: None,
            content: Some("test".into()),
            components: None,
            embeds: Vec::new(),
            flags: Some(MessageFlags::EPHEMERAL),
            tts: None,
        });

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "InteractionResponse",
                    len: 2,
                },
                Token::Str("type"),
                Token::U8(4),
                Token::Str("data"),
                Token::Struct {
                    name: "CallbackData",
                    len: 2,
                },
                Token::Str("content"),
                Token::Some,
                Token::Str("test"),
                Token::Str("flags"),
                Token::Some,
                Token::U64(64),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
