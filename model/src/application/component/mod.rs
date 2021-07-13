//! Types for Message [`Component`] support.
//!
//! Message components are a Discord API framework for adding interactive
//! elements to created [`Message`]s.
//!
//! Refer to [Discord Docs/Message Components] for additional information.
//!
//! [`Message`]: crate::channel::Message
//! [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components

pub mod action_row;
pub mod button;
pub mod select_menu;

mod kind;

pub use self::{
    action_row::ActionRow, button::Button, kind::ComponentType, select_menu::SelectMenu,
};

use serde::{Deserialize, Serialize};

/// Interactive element of a message that an application uses.
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#what-are-components
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
}

impl Component {
    /// Type of component that this is.
    ///
    /// ```
    /// use twilight_model::application::component::{
    ///     button::{ButtonStyle, Button},
    ///     ComponentType,
    ///     Component,
    /// };
    ///
    /// let component = Component::Button(Button {
    ///     custom_id: None,
    ///     disabled: false,
    ///     emoji: None,
    ///     label: Some("ping".to_owned()),
    ///     style: ButtonStyle::Primary,
    ///     url: None,
    /// });
    ///
    /// assert_eq!(ComponentType::Button, component.kind());
    /// ```
    pub const fn kind(&self) -> ComponentType {
        match self {
            Self::ActionRow(_) => ComponentType::ActionRow,
            Self::Button(_) => ComponentType::Button,
            Self::SelectMenu(_) => ComponentType::SelectMenu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        button::{Button, ButtonStyle},
        select_menu::{SelectMenu, SelectMenuOption},
        ActionRow, Component, ComponentType,
    };
    use serde_test::Token;

    #[test]
    fn test_component_full() {
        let component = Component::ActionRow(ActionRow {
            components: Vec::from([
                Component::Button(Button {
                    style: ButtonStyle::Primary,
                    emoji: None,
                    label: Some("test label".into()),
                    custom_id: Some("test custom id".into()),
                    url: None,
                    disabled: false,
                }),
                Component::SelectMenu(SelectMenu {
                    custom_id: "test custom id 2".into(),
                    placeholder: Some("test placeholder".into()),
                    min_values: Some(5),
                    max_values: Some(25),
                    options: Vec::from([SelectMenuOption {
                        label: "test option label".into(),
                        value: "test option value".into(),
                        description: Some("test description".into()),
                        emoji: None,
                        default: false,
                    }]),
                }),
            ]),
        });

        serde_test::assert_tokens(
            &component,
            &[
                Token::Struct {
                    name: "ActionRow",
                    len: 2,
                },
                Token::Str("components"),
                Token::Seq { len: Some(2) },
                Token::Struct {
                    name: "Button",
                    len: 5,
                },
                Token::Str("custom_id"),
                Token::Some,
                Token::Str("test custom id"),
                Token::Str("disabled"),
                Token::Bool(false),
                Token::Str("label"),
                Token::Some,
                Token::Str("test label"),
                Token::Str("style"),
                Token::U8(ButtonStyle::Primary as u8),
                Token::Str("type"),
                Token::U8(ComponentType::Button as u8),
                Token::StructEnd,
                Token::Struct {
                    name: "SelectMenu",
                    len: 6,
                },
                Token::Str("custom_id"),
                Token::Str("test custom id 2"),
                Token::Str("max_values"),
                Token::Some,
                Token::U8(25),
                Token::Str("min_values"),
                Token::Some,
                Token::U8(5),
                Token::Str("options"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "SelectMenuOption",
                    len: 4,
                },
                Token::Str("default"),
                Token::Bool(false),
                Token::Str("description"),
                Token::Some,
                Token::Str("test description"),
                Token::Str("label"),
                Token::Str("test option label"),
                Token::Str("value"),
                Token::Str("test option value"),
                Token::StructEnd,
                Token::SeqEnd,
                Token::Str("placeholder"),
                Token::Some,
                Token::Str("test placeholder"),
                Token::Str("type"),
                Token::U8(ComponentType::SelectMenu as u8),
                Token::StructEnd,
                Token::SeqEnd,
                Token::Str("type"),
                Token::U8(ComponentType::ActionRow as u8),
                Token::StructEnd,
            ],
        );
    }
}
