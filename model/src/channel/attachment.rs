use crate::id::AttachmentId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Attachment {
    /// Attachment's [media type].
    ///
    /// [media type]: https://en.wikipedia.org/wiki/Media_type
    pub content_type: Option<String>,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    pub id: AttachmentId,
    pub proxy_url: String,
    pub size: u64,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::Attachment;
    use crate::id::AttachmentId;
    use serde_test::Token;

    #[test]
    fn test_attachment() {
        let value = Attachment {
            content_type: Some("image/png".to_owned()),
            filename: "a.png".to_owned(),
            height: Some(184),
            id: AttachmentId(700_000_000_000_000_000),
            proxy_url: "https://cdn.example.com/1.png".to_owned(),
            size: 13_593,
            url: "https://example.com/1.png".to_owned(),
            width: Some(184),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "Attachment",
                    len: 8,
                },
                Token::Str("content_type"),
                Token::Some,
                Token::Str("image/png"),
                Token::Str("filename"),
                Token::Str("a.png"),
                Token::Str("height"),
                Token::Some,
                Token::U64(184),
                Token::Str("id"),
                Token::NewtypeStruct {
                    name: "AttachmentId",
                },
                Token::Str("700000000000000000"),
                Token::Str("proxy_url"),
                Token::Str("https://cdn.example.com/1.png"),
                Token::Str("size"),
                Token::U64(13_593),
                Token::Str("url"),
                Token::Str("https://example.com/1.png"),
                Token::Str("width"),
                Token::Some,
                Token::U64(184),
                Token::StructEnd,
            ],
        );
    }
}
