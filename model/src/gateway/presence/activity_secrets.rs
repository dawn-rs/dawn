use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ActivitySecrets {
    pub join: Option<String>,
    #[serde(rename = "match")]
    pub match_: Option<String>,
    pub spectate: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::ActivitySecrets;
    use serde_test::Token;

    #[test]
    fn test_activity_secrets() {
        let value = ActivitySecrets {
            join: Some("a".to_owned()),
            match_: Some("b".to_owned()),
            spectate: Some("c".to_owned()),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "ActivitySecrets",
                    len: 3,
                },
                Token::Str("join"),
                Token::Some,
                Token::Str("a"),
                Token::Str("match"),
                Token::Some,
                Token::Str("b"),
                Token::Str("spectate"),
                Token::Some,
                Token::Str("c"),
                Token::StructEnd,
            ],
        );
    }
}
