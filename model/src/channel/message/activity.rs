use super::MessageActivityType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MessageActivity {
    #[serde(rename = "type")]
    pub kind: MessageActivityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{MessageActivity, MessageActivityType};
    use serde_test::Token;

    #[test]
    fn test_message_activity() {
        let value = MessageActivity {
            kind: MessageActivityType::Join,
            party_id: Some("test".to_owned()),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "MessageActivity",
                    len: 2,
                },
                Token::Str("type"),
                Token::U8(1),
                Token::Str("party_id"),
                Token::Some,
                Token::Str("test"),
                Token::StructEnd,
            ],
        );
    }
}
