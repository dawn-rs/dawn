/// Contains all of the input validation functions for requests.
///
/// This is in a centralised place so that the validation parameters can be kept
/// up-to-date more easily and because some of the checks are re-used across
/// different modules.
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::channel::embed::Embed;

/// An embed is not valid.
///
/// Referenced values are used from [the Discord docs][docs].
///
/// [docs]: https://discord.com/developers/docs/resources/channel#embed-limits
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum EmbedValidationError {
    /// The embed author's name is larger than
    /// [the maximum][`AUTHOR_NAME_LENGTH`].
    ///
    /// [`AUTHOR_NAME_LENGTH`]: #const.AUTHOR_NAME_LENGTH
    AuthorNameTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The embed description is larger than
    /// [the maximum][`DESCRIPTION_LENGTH`].
    ///
    /// [`DESCRIPTION_LENGTH`]: #const.DESCRIPTION_LENGTH
    DescriptionTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The combined content of all embed fields - author name, description,
    /// footer, field names and values, and title - is larger than
    /// [the maximum][`EMBED_TOTAL_LENGTH`].
    ///
    /// [`EMBED_TOTAL_LENGTH`]: #const.EMBED_TOTAL_LENGTH
    EmbedTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// A field's name is larger than [the maximum][`FIELD_NAME_LENGTH`].
    ///
    /// [`FIELD_NAME_LENGTH`]: #const.FIELD_NAME_LENGTH
    FieldNameTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// A field's value is larger than [the maximum][`FIELD_VALUE_LENGTH`].
    ///
    /// [`FIELD_VALUE_LENGTH`]: #const.FIELD_VALUE_LENGTH
    FieldValueTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The footer text is larger than [the maximum][`FOOTER_TEXT_LENGTH`].
    ///
    /// [`FOOTER_TEXT_LENGTH`]: #const.FOOTER_TEXT_LENGTH
    FooterTextTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The title is larger than [the maximum][`TITLE_LENGTH`].
    ///
    /// [`TITLE_LENGTH`]: #const.TITLE_LENGTH
    TitleTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// There are more than [the maximum][`FIELD_COUNT`] number of fields in the
    /// embed.
    ///
    /// [`FIELD_COUNT`]: #const.FIELD_COUNT
    TooManyFields {
        /// The number of fields that were provided.
        amount: usize,
    },
}

impl EmbedValidationError {
    /// The maximum embed author name length in codepoints.
    pub const AUTHOR_NAME_LENGTH: usize = 256;

    /// The maximum embed description length in codepoints.
    pub const DESCRIPTION_LENGTH: usize = 2048;

    /// The maximum combined embed length in codepoints.
    pub const EMBED_TOTAL_LENGTH: usize = 6000;

    /// The maximum number of fields in an embed.
    pub const FIELD_COUNT: usize = 25;

    /// The maximum length of an embed field name in codepoints.
    pub const FIELD_NAME_LENGTH: usize = 256;

    /// The maximum length of an embed field value in codepoints.
    pub const FIELD_VALUE_LENGTH: usize = 1024;

    /// The maximum embed footer length in codepoints.
    pub const FOOTER_TEXT_LENGTH: usize = 2048;

    /// The maximum embed title length in codepoints.
    pub const TITLE_LENGTH: usize = 256;
}

impl Display for EmbedValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::AuthorNameTooLarge { chars } => write!(
                f,
                "the author name is {} characters long, but the max is {}",
                chars,
                Self::AUTHOR_NAME_LENGTH
            ),
            Self::DescriptionTooLarge { chars } => write!(
                f,
                "the description is {} characters long, but the max is {}",
                chars,
                Self::DESCRIPTION_LENGTH
            ),
            Self::EmbedTooLarge { chars } => write!(
                f,
                "the combined total length of the embed is {} characters long, but the max is {}",
                chars,
                Self::EMBED_TOTAL_LENGTH
            ),
            Self::FieldNameTooLarge { chars } => write!(
                f,
                "a field name is {} characters long, but the max is {}",
                chars,
                Self::FIELD_NAME_LENGTH
            ),
            Self::FieldValueTooLarge { chars } => write!(
                f,
                "a field value is {} characters long, but the max is {}",
                chars,
                Self::FIELD_VALUE_LENGTH
            ),
            Self::FooterTextTooLarge { chars } => write!(
                f,
                "the footer's text is {} characters long, but the max is {}",
                chars,
                Self::FOOTER_TEXT_LENGTH
            ),
            Self::TitleTooLarge { chars } => write!(
                f,
                "the title's length is {} characters long, but the max is {}",
                chars,
                Self::TITLE_LENGTH
            ),
            Self::TooManyFields { amount } => write!(
                f,
                "there are {} fields, but the maximum amount is {}",
                amount,
                Self::FIELD_COUNT
            ),
        }
    }
}

impl Error for EmbedValidationError {}

pub fn ban_delete_message_days(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#create-guild-ban-query-string-params>
    value <= 7
}

pub fn channel_name(value: impl AsRef<str>) -> bool {
    _channel_name(value.as_ref())
}

fn _channel_name(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure>
    len >= 2 && len <= 100
}

pub fn content_limit(value: impl AsRef<str>) -> bool {
    _content_limit(value.as_ref())
}

fn _content_limit(value: &str) -> bool {
    // <https://discordapp.com/developers/docs/resources/channel#create-message-params>
    value.chars().count() <= 2000
}

pub fn embed(embed: &Embed) -> Result<(), EmbedValidationError> {
    let mut total = 0;

    if embed.fields.len() > EmbedValidationError::FIELD_COUNT {
        return Err(EmbedValidationError::TooManyFields {
            amount: embed.fields.len(),
        });
    }

    if let Some(name) = embed
        .author
        .as_ref()
        .and_then(|author| author.name.as_ref())
    {
        let chars = name.chars().count();

        if chars > EmbedValidationError::AUTHOR_NAME_LENGTH {
            return Err(EmbedValidationError::AuthorNameTooLarge { chars });
        }

        total += chars;
    }

    if let Some(description) = embed.description.as_ref() {
        let chars = description.chars().count();

        if chars > EmbedValidationError::DESCRIPTION_LENGTH {
            return Err(EmbedValidationError::DescriptionTooLarge { chars });
        }

        total += chars;
    }

    if let Some(footer) = embed.footer.as_ref() {
        let chars = footer.text.chars().count();

        if chars > EmbedValidationError::FOOTER_TEXT_LENGTH {
            return Err(EmbedValidationError::FooterTextTooLarge { chars });
        }

        total += chars;
    }

    for field in &embed.fields {
        let name_chars = field.name.chars().count();

        if name_chars > EmbedValidationError::FIELD_NAME_LENGTH {
            return Err(EmbedValidationError::FieldNameTooLarge { chars: name_chars });
        }

        let value_chars = field.value.chars().count();

        if value_chars > EmbedValidationError::FIELD_VALUE_LENGTH {
            return Err(EmbedValidationError::FieldValueTooLarge { chars: value_chars });
        }

        total += name_chars + value_chars;
    }

    if let Some(title) = embed.title.as_ref() {
        let chars = title.chars().count();

        if chars > EmbedValidationError::TITLE_LENGTH {
            return Err(EmbedValidationError::TitleTooLarge { chars });
        }

        total += chars;
    }

    if total > EmbedValidationError::EMBED_TOTAL_LENGTH {
        return Err(EmbedValidationError::EmbedTooLarge { chars: total });
    }

    Ok(())
}

pub fn get_audit_log_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/audit-log#get-guild-audit-log-query-string-parameters>
    value > 0 && value <= 100
}

pub fn get_channel_messages_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/channel#get-channel-messages-query-string-params>
    value > 0 && value <= 100
}

pub fn get_current_user_guilds_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/user#get-current-user-guilds-query-string-params>
    value > 0 && value <= 100
}

pub fn get_guild_members_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#list-guild-members-query-string-params>
    value > 0 && value <= 1000
}

pub fn get_reactions_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/channel#get-reactions-query-string-params>
    value > 0 && value <= 100
}

pub fn guild_name(value: impl AsRef<str>) -> bool {
    _guild_name(value.as_ref())
}

fn _guild_name(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/guild#guild-object-guild-structure>
    len >= 2 && len <= 100
}

pub fn guild_prune_days(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#get-guild-prune-count-query-string-params>
    value > 0
}

pub fn nickname(value: impl AsRef<str>) -> bool {
    _nickname(value.as_ref())
}

fn _nickname(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/user#usernames-and-nicknames>
    len > 0 && len <= 32
}

pub fn username(value: impl AsRef<str>) -> bool {
    // <https://discordapp.com/developers/docs/resources/user#usernames-and-nicknames>
    _username(value.as_ref())
}

fn _username(value: &str) -> bool {
    let len = value.chars().count();

    len >= 2 && len <= 32
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::channel::embed::{EmbedAuthor, EmbedField, EmbedFooter};

    fn base_embed() -> Embed {
        Embed {
            author: None,
            color: None,
            description: None,
            fields: Vec::new(),
            footer: None,
            image: None,
            kind: "rich".to_owned(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: None,
            url: None,
            video: None,
        }
    }

    #[test]
    fn test_ban_delete_message_days() {
        assert!(ban_delete_message_days(0));
        assert!(ban_delete_message_days(1));
        assert!(ban_delete_message_days(7));

        assert!(!ban_delete_message_days(8));
    }

    #[test]
    fn test_channel_name() {
        assert!(channel_name("aa"));
        assert!(channel_name("a".repeat(100)));

        assert!(!channel_name(""));
        assert!(!channel_name("a"));
        assert!(!channel_name("a".repeat(101)));
    }

    #[test]
    fn test_content_limit() {
        assert!(content_limit(""));
        assert!(content_limit("a".repeat(2000)));

        assert!(!content_limit("a".repeat(2001)));
    }

    #[test]
    fn test_embed_base() {
        let embed = base_embed();

        assert!(super::embed(&embed).is_ok());
    }

    #[test]
    fn test_embed_normal() {
        let mut embed = base_embed();
        embed.author.replace(EmbedAuthor {
            icon_url: None,
            name: Some("twilight".to_owned()),
            proxy_icon_url: None,
            url: None,
        });
        embed.color.replace(0xff_00_00);
        embed.description.replace("a".repeat(100));
        embed.fields.push(EmbedField {
            inline: true,
            name: "b".repeat(25),
            value: "c".repeat(200),
        });
        embed.title.replace("this is a normal title".to_owned());

        assert!(super::embed(&embed).is_ok());
    }

    #[test]
    fn test_embed_author_name_limit() {
        let mut embed = base_embed();
        embed.author.replace(EmbedAuthor {
            icon_url: None,
            name: Some(str::repeat("a", 256)),
            proxy_icon_url: None,
            url: None,
        });
        assert!(super::embed(&embed).is_ok());

        embed.author.replace(EmbedAuthor {
            icon_url: None,
            name: Some(str::repeat("a", 257)),
            proxy_icon_url: None,
            url: None,
        });
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::AuthorNameTooLarge { chars: 257 })
        ));
    }

    #[test]
    fn test_embed_description_limit() {
        let mut embed = base_embed();
        embed.description.replace(str::repeat("a", 2048));
        assert!(super::embed(&embed).is_ok());

        embed.description.replace(str::repeat("a", 2049));
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::DescriptionTooLarge { chars: 2049 })
        ));
    }

    #[test]
    fn test_embed_field_count_limit() {
        let mut embed = base_embed();

        for _ in 0..26 {
            embed.fields.push(EmbedField {
                inline: true,
                name: "a".to_owned(),
                value: "a".to_owned(),
            });
        }

        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::TooManyFields { amount: 26 })
        ));
    }

    #[test]
    fn test_embed_field_name_limit() {
        let mut embed = base_embed();
        embed.fields.push(EmbedField {
            inline: true,
            name: str::repeat("a", 256),
            value: "a".to_owned(),
        });
        assert!(super::embed(&embed).is_ok());

        embed.fields.push(EmbedField {
            inline: true,
            name: str::repeat("a", 257),
            value: "a".to_owned(),
        });
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::FieldNameTooLarge { chars: 257 })
        ));
    }

    #[test]
    fn test_embed_field_value_limit() {
        let mut embed = base_embed();
        embed.fields.push(EmbedField {
            inline: true,
            name: "a".to_owned(),
            value: str::repeat("a", 1024),
        });
        assert!(super::embed(&embed).is_ok());

        embed.fields.push(EmbedField {
            inline: true,
            name: "a".to_owned(),
            value: str::repeat("a", 1025),
        });
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::FieldValueTooLarge { chars: 1025 })
        ));
    }

    #[test]
    fn test_embed_footer_text_limit() {
        let mut embed = base_embed();
        embed.footer.replace(EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: str::repeat("a", 2048),
        });
        assert!(super::embed(&embed).is_ok());

        embed.footer.replace(EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: str::repeat("a", 2049),
        });
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::FooterTextTooLarge { chars: 2049 })
        ));
    }

    #[test]
    fn test_embed_title_limit() {
        let mut embed = base_embed();
        embed.title.replace(str::repeat("a", 256));
        assert!(super::embed(&embed).is_ok());

        embed.title.replace(str::repeat("a", 257));
        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::TitleTooLarge { chars: 257 })
        ));
    }

    #[test]
    fn test_embed_combined_limit() {
        let mut embed = base_embed();
        embed.description.replace(str::repeat("a", 2048));
        embed.title.replace(str::repeat("a", 256));

        for _ in 0..5 {
            embed.fields.push(EmbedField {
                inline: true,
                name: str::repeat("a", 100),
                value: str::repeat("a", 500),
            })
        }

        // we're at 5304 characters now
        assert!(super::embed(&embed).is_ok());

        embed.footer.replace(EmbedFooter {
            icon_url: None,
            proxy_icon_url: None,
            text: str::repeat("a", 1000),
        });

        assert!(matches!(
            super::embed(&embed),
            Err(EmbedValidationError::EmbedTooLarge { chars: 6304 })
        ));
    }

    #[test]
    fn test_get_audit_log_limit() {
        assert!(get_audit_log_limit(1));
        assert!(get_audit_log_limit(100));

        assert!(!get_audit_log_limit(0));
        assert!(!get_audit_log_limit(101));
    }

    #[test]
    fn test_get_channels_limit() {
        assert!(get_channel_messages_limit(1));
        assert!(get_channel_messages_limit(100));

        assert!(!get_channel_messages_limit(0));
        assert!(!get_channel_messages_limit(101));
    }

    #[test]
    fn test_get_current_user_guilds_limit() {
        assert!(get_current_user_guilds_limit(1));
        assert!(get_current_user_guilds_limit(100));

        assert!(!get_current_user_guilds_limit(0));
        assert!(!get_current_user_guilds_limit(101));
    }

    #[test]
    fn test_get_guild_members_limit() {
        assert!(get_guild_members_limit(1));
        assert!(get_guild_members_limit(1000));

        assert!(!get_guild_members_limit(0));
        assert!(!get_guild_members_limit(1001));
    }

    #[test]
    fn test_get_reactions_limit() {
        assert!(get_reactions_limit(1));
        assert!(get_reactions_limit(100));

        assert!(!get_reactions_limit(0));
        assert!(!get_reactions_limit(101));
    }

    #[test]
    fn test_guild_name() {
        assert!(guild_name("aa"));
        assert!(guild_name("a".repeat(100)));

        assert!(!guild_name(""));
        assert!(!guild_name("a"));
        assert!(!guild_name("a".repeat(101)));
    }

    #[test]
    fn test_guild_prune_days() {
        assert!(!guild_prune_days(0));
        assert!(guild_prune_days(1));
        assert!(guild_prune_days(100));
    }

    #[test]
    fn test_nickname() {
        assert!(nickname("a"));
        assert!(nickname("a".repeat(32)));

        assert!(!nickname(""));
        assert!(!nickname("a".repeat(33)));
    }

    #[test]
    fn test_username() {
        assert!(username("aa"));
        assert!(username("a".repeat(32)));

        assert!(!username("a"));
        assert!(!username("a".repeat(33)));
    }
}
