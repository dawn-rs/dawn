use crate::channel::ReactionType;

use super::ComponentType;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Clickable interactive components that render on messages.
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#button-object-button-structure
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Button {
    /// User defined identifier for the button.
    ///
    /// This field is required when using the following [`ButtonStyle`]s:
    ///
    /// - [`ButtonStyle::Danger`]
    /// - [`ButtonStyle::Primary`]
    /// - [`ButtonStyle::Secondary`]
    /// - [`ButtonStyle::Success`]
    pub custom_id: Option<String>,
    /// Whether the button is disabled.
    ///
    /// Defaults to `false`.
    #[serde(default)]
    pub disabled: bool,
    /// Visual emoji for clients to display with the button.
    pub emoji: Option<ReactionType>,
    /// Text appearing on the button.
    pub label: Option<String>,
    /// Style variant of the button.
    pub style: ButtonStyle,
    /// URL for buttons of a [`ButtonStyle::Link`] style.
    pub url: Option<String>,
}

/// Style of a [`Button`].
///
/// Refer to [the discord docs] for additional information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/message-components#button-object-button-styles
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Hash, PartialEq, PartialOrd, Serialize_repr)]
#[repr(u8)]
pub enum ButtonStyle {
    /// Button indicates a primary action.
    ///
    /// Selecting this button style requires specifying the
    /// [`Button::custom_id`] field.
    Primary = 1,
    /// Button indicates a secondary action.
    ///
    /// Selecting this button style requires specifying the
    /// [`Button::custom_id`] field.
    Secondary = 2,
    /// Button indicates a successful action.
    ///
    /// Selecting this button style requires specifying the
    /// [`Button::custom_id`] field.
    Success = 3,
    /// Button indicates a dangerous action.
    ///
    /// Selecting this button style requires specifying the
    /// [`Button::custom_id`] field.
    Danger = 4,
    /// Button indicates an action with a link.
    ///
    /// Selecting this button style requires specifying the [`Button::url`]
    /// field.
    Link = 5,
}

impl Serialize for Button {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Base of 3 to account for the fields that are always present:
        //
        // - `disabled`
        // - `style`
        // - `type`
        let field_count = 3
            + usize::from(self.custom_id.is_some())
            + usize::from(self.emoji.is_some())
            + usize::from(self.label.is_some())
            + usize::from(self.url.is_some());
        let mut state = serializer.serialize_struct("Button", field_count)?;

        if self.custom_id.is_some() {
            state.serialize_field("custom_id", &self.custom_id)?;
        }

        state.serialize_field("disabled", &self.disabled)?;

        if self.emoji.is_some() {
            state.serialize_field("emoji", &self.emoji)?;
        }

        if self.label.is_some() {
            state.serialize_field("label", &self.label)?;
        }

        state.serialize_field("style", &self.style)?;
        state.serialize_field("type", &ComponentType::Button)?;

        if self.url.is_some() {
            state.serialize_field("url", &self.url)?;
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    // Required due to the use of a unicode emoji in a constant.
    #![allow(clippy::non_ascii_literal)]

    use super::{Button, ButtonStyle};
    use crate::{application::component::ComponentType, channel::ReactionType};
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_fields, assert_impl_all, const_assert_eq};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(Button: custom_id, disabled, emoji, label, style, url);
    assert_impl_all!(
        ButtonStyle: Clone,
        Copy,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        PartialOrd,
        Send,
        Serialize,
        Sync
    );
    assert_impl_all!(
        Button: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Send,
        Serialize,
        Sync
    );
    const_assert_eq!(1, ButtonStyle::Primary as u8);
    const_assert_eq!(2, ButtonStyle::Secondary as u8);
    const_assert_eq!(3, ButtonStyle::Success as u8);
    const_assert_eq!(4, ButtonStyle::Danger as u8);
    const_assert_eq!(5, ButtonStyle::Link as u8);

    #[test]
    fn test_button_style() {
        serde_test::assert_tokens(&ButtonStyle::Primary, &[Token::U8(1)]);
        serde_test::assert_tokens(&ButtonStyle::Secondary, &[Token::U8(2)]);
        serde_test::assert_tokens(&ButtonStyle::Success, &[Token::U8(3)]);
        serde_test::assert_tokens(&ButtonStyle::Danger, &[Token::U8(4)]);
        serde_test::assert_tokens(&ButtonStyle::Link, &[Token::U8(5)]);
    }

    #[test]
    fn test_button() {
        // Free Palestine.
        //
        // Palestinian Flag.
        const FLAG: &str = "🇵🇸";

        let value = Button {
            custom_id: Some("test".to_owned()),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: FLAG.to_owned(),
            }),
            label: Some("Test".to_owned()),
            style: ButtonStyle::Link,
            url: Some("https://twilight.rs".to_owned()),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Button",
                    len: 7,
                },
                Token::String("custom_id"),
                Token::Some,
                Token::String("test"),
                Token::String("disabled"),
                Token::Bool(false),
                Token::String("emoji"),
                Token::Some,
                Token::Struct {
                    name: "ReactionType",
                    len: 1,
                },
                Token::String("name"),
                Token::String(FLAG),
                Token::StructEnd,
                Token::String("label"),
                Token::Some,
                Token::String("Test"),
                Token::String("style"),
                Token::U8(ButtonStyle::Link as u8),
                Token::String("type"),
                Token::U8(ComponentType::Button as u8),
                Token::String("url"),
                Token::Some,
                Token::String("https://twilight.rs"),
                Token::StructEnd,
            ],
        );
    }
}
