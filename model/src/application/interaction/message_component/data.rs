use crate::application::component::ComponentType;
use serde::{Deserialize, Serialize};

/// Data received when an [`MessageComponent`] interaction is executed.
///
/// Refer to [the discord docs] for more information.
///
/// [`MessageComponent`]: crate::application::interaction::Interaction::MessageComponent
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MessageComponentInteractionData {
    pub custom_id: String,
    pub component_type: ComponentType,
    #[serde(default)]
    pub values: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::MessageComponentInteractionData;
    use crate::application::component::ComponentType;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(
        MessageComponentInteractionData: custom_id,
        component_type,
        values
    );
    assert_impl_all!(
        MessageComponentInteractionData: Clone,
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
    fn test_message_component_interaction_data() {
        let value = MessageComponentInteractionData {
            custom_id: "test".to_owned(),
            component_type: ComponentType::Button,
            values: Vec::from(["1".to_owned(), "2".to_owned()]),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "MessageComponentInteractionData",
                    len: 3,
                },
                Token::String("custom_id"),
                Token::String("test"),
                Token::String("component_type"),
                Token::U8(ComponentType::Button as u8),
                Token::String("values"),
                Token::Seq { len: Some(2) },
                Token::String("1"),
                Token::String("2"),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        )
    }
}
