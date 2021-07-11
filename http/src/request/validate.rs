/// Contains all of the input validation functions for requests.
///
/// This is in a centralised place so that the validation parameters can be kept
/// up-to-date more easily and because some of the checks are re-used across
/// different modules.
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::channel::{embed::Embed, ChannelType};

/// An embed is not valid.
///
/// Referenced values are used from [the Discord docs][docs].
///
/// [docs]: https://discord.com/developers/docs/resources/channel#embed-limits
#[derive(Debug)]
pub struct EmbedValidationError {
    kind: EmbedValidationErrorType,
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

    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &EmbedValidationErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        EmbedValidationErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for EmbedValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            EmbedValidationErrorType::AuthorNameTooLarge { chars } => {
                f.write_str("the author name is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::AUTHOR_NAME_LENGTH, f)
            }
            EmbedValidationErrorType::DescriptionTooLarge { chars } => {
                f.write_str("the description is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::DESCRIPTION_LENGTH, f)
            }
            EmbedValidationErrorType::EmbedTooLarge { chars } => {
                f.write_str("the combined total length of the embed is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::EMBED_TOTAL_LENGTH, f)
            }
            EmbedValidationErrorType::FieldNameTooLarge { chars } => {
                f.write_str("a field name is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::FIELD_NAME_LENGTH, f)
            }
            EmbedValidationErrorType::FieldValueTooLarge { chars } => {
                f.write_str("a field value is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::FIELD_VALUE_LENGTH, f)
            }
            EmbedValidationErrorType::FooterTextTooLarge { chars } => {
                f.write_str("the footer's text is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::FOOTER_TEXT_LENGTH, f)
            }
            EmbedValidationErrorType::TitleTooLarge { chars } => {
                f.write_str("the title's length is ")?;
                Display::fmt(chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::TITLE_LENGTH, f)
            }
            EmbedValidationErrorType::TooManyFields { amount } => {
                f.write_str("there are ")?;
                Display::fmt(amount, f)?;
                f.write_str(" fields, but the maximum amount is ")?;

                Display::fmt(&Self::FIELD_COUNT, f)
            }
        }
    }
}

impl Error for EmbedValidationError {}

/// Type of [`EmbedValidationError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum EmbedValidationErrorType {
    /// The embed author's name is larger than
    /// [the maximum][`AUTHOR_NAME_LENGTH`].
    ///
    /// [`AUTHOR_NAME_LENGTH`]: Self::AUTHOR_NAME_LENGTH
    AuthorNameTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The embed description is larger than
    /// [the maximum][`DESCRIPTION_LENGTH`].
    ///
    /// [`DESCRIPTION_LENGTH`]: Self::DESCRIPTION_LENGTH
    DescriptionTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The combined content of all embed fields - author name, description,
    /// footer, field names and values, and title - is larger than
    /// [the maximum][`EMBED_TOTAL_LENGTH`].
    ///
    /// [`EMBED_TOTAL_LENGTH`]: Self::EMBED_TOTAL_LENGTH
    EmbedTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// A field's name is larger than [the maximum][`FIELD_NAME_LENGTH`].
    ///
    /// [`FIELD_NAME_LENGTH`]: Self::FIELD_NAME_LENGTH
    FieldNameTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// A field's value is larger than [the maximum][`FIELD_VALUE_LENGTH`].
    ///
    /// [`FIELD_VALUE_LENGTH`]: Self::FIELD_VALUE_LENGTH
    FieldValueTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The footer text is larger than [the maximum][`FOOTER_TEXT_LENGTH`].
    ///
    /// [`FOOTER_TEXT_LENGTH`]: Self::FOOTER_TEXT_LENGTH
    FooterTextTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// The title is larger than [the maximum][`TITLE_LENGTH`].
    ///
    /// [`TITLE_LENGTH`]: Self::TITLE_LENGTH
    TitleTooLarge {
        /// The number of codepoints that were provided.
        chars: usize,
    },
    /// There are more than [the maximum][`FIELD_COUNT`] number of fields in the
    /// embed.
    ///
    /// [`FIELD_COUNT`]: Self::FIELD_COUNT
    TooManyFields {
        /// The number of fields that were provided.
        amount: usize,
    },
}

pub const fn ban_delete_message_days(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#create-guild-ban-query-string-params>
    value <= 7
}

pub fn channel_name(value: impl AsRef<str>) -> bool {
    _channel_name(value.as_ref())
}

fn _channel_name(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure>
    (1..=100).contains(&len)
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
        return Err(EmbedValidationError {
            kind: EmbedValidationErrorType::TooManyFields {
                amount: embed.fields.len(),
            },
        });
    }

    if let Some(name) = embed
        .author
        .as_ref()
        .and_then(|author| author.name.as_ref())
    {
        let chars = name.chars().count();

        if chars > EmbedValidationError::AUTHOR_NAME_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::AuthorNameTooLarge { chars },
            });
        }

        total += chars;
    }

    if let Some(description) = embed.description.as_ref() {
        let chars = description.chars().count();

        if chars > EmbedValidationError::DESCRIPTION_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::DescriptionTooLarge { chars },
            });
        }

        total += chars;
    }

    if let Some(footer) = embed.footer.as_ref() {
        let chars = footer.text.chars().count();

        if chars > EmbedValidationError::FOOTER_TEXT_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::FooterTextTooLarge { chars },
            });
        }

        total += chars;
    }

    for field in &embed.fields {
        let name_chars = field.name.chars().count();

        if name_chars > EmbedValidationError::FIELD_NAME_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::FieldNameTooLarge { chars: name_chars },
            });
        }

        let value_chars = field.value.chars().count();

        if value_chars > EmbedValidationError::FIELD_VALUE_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::FieldValueTooLarge { chars: value_chars },
            });
        }

        total += name_chars + value_chars;
    }

    if let Some(title) = embed.title.as_ref() {
        let chars = title.chars().count();

        if chars > EmbedValidationError::TITLE_LENGTH {
            return Err(EmbedValidationError {
                kind: EmbedValidationErrorType::TitleTooLarge { chars },
            });
        }

        total += chars;
    }

    if total > EmbedValidationError::EMBED_TOTAL_LENGTH {
        return Err(EmbedValidationError {
            kind: EmbedValidationErrorType::EmbedTooLarge { chars: total },
        });
    }

    Ok(())
}

pub const fn get_audit_log_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/audit-log#get-guild-audit-log-query-string-parameters>
    value >= 1 && value <= 100
}

pub const fn get_channel_messages_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/channel#get-channel-messages-query-string-params>
    value >= 1 && value <= 100
}

pub const fn get_current_user_guilds_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/user#get-current-user-guilds-query-string-params>
    value >= 1 && value <= 100
}

pub const fn get_guild_members_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#list-guild-members-query-string-params>
    value >= 1 && value <= 1000
}

pub const fn search_guild_members_limit(value: u64) -> bool {
    value > 0 && value <= 1000
}

pub const fn get_reactions_limit(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/channel#get-reactions-query-string-params>
    value >= 1 && value <= 100
}

pub fn guild_name(value: impl AsRef<str>) -> bool {
    _guild_name(value.as_ref())
}

fn _guild_name(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/guild#guild-object-guild-structure>
    (2..=100).contains(&len)
}

pub const fn guild_prune_days(value: u64) -> bool {
    // <https://discordapp.com/developers/docs/resources/guild#get-guild-prune-count-query-string-params>
    value > 0 && value <= 30
}

pub const fn invite_max_age(value: u64) -> bool {
    // <https://discord.com/developers/docs/resources/channel#create-channel-invite-json-params>
    value <= 604_800
}

pub const fn invite_max_uses(value: u64) -> bool {
    // <https://discord.com/developers/docs/resources/channel#create-channel-invite-json-params>
    value <= 100
}

pub fn nickname(value: impl AsRef<str>) -> bool {
    _nickname(value.as_ref())
}

fn _nickname(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/user#usernames-and-nicknames>
    (1..=32).contains(&len)
}

pub fn username(value: impl AsRef<str>) -> bool {
    // <https://discordapp.com/developers/docs/resources/user#usernames-and-nicknames>
    _username(value.as_ref())
}

fn _username(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discordapp.com/developers/docs/resources/user#usernames-and-nicknames>
    (2..=32).contains(&len)
}

pub fn template_name(value: impl AsRef<str>) -> bool {
    _template_name(value.as_ref())
}

fn _template_name(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discord.com/developers/docs/resources/template#create-guild-template-json-params>
    (1..=100).contains(&len)
}

pub fn template_description(value: impl AsRef<str>) -> bool {
    _template_name(value.as_ref())
}

fn _template_description(value: &str) -> bool {
    let len = value.chars().count();

    // <https://discord.com/developers/docs/resources/template#create-guild-template-json-params>
    (0..=120).contains(&len)
}

pub fn stage_topic(value: impl AsRef<str>) -> bool {
    _stage_topic(value.as_ref())
}

fn _stage_topic(value: &str) -> bool {
    let len = value.chars().count();

    // <https://github.com/discord/discord-api-docs/commit/f019fc358047050513c623f3639b6e96809f9280>
    (0..=120).contains(&len)
}

pub fn command_name(value: impl AsRef<str>) -> bool {
    _command_name(value.as_ref())
}

fn _command_name(value: &str) -> bool {
    let len = value.chars().count();

    // https://discord.com/developers/docs/interactions/slash-commands#applicationcommandoption
    (3..=32).contains(&len)
}

pub fn command_description(value: impl AsRef<str>) -> bool {
    _command_description(value.as_ref())
}

fn _command_description(value: &str) -> bool {
    let len = value.chars().count();

    // https://discord.com/developers/docs/interactions/slash-commands#applicationcommandoption
    (1..=100).contains(&len)
}

pub fn command_permissions(len: usize) -> bool {
    // https://discord.com/developers/docs/interactions/slash-commands#edit-application-command-permissions
    (0..=10).contains(&len)
}

pub const fn is_thread(channel_type: ChannelType) -> bool {
    matches!(
        channel_type,
        ChannelType::GuildNewsThread
            | ChannelType::GuildPublicThread
            | ChannelType::GuildPrivateThread
    )
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
        assert!(channel_name("a"));
        assert!(channel_name("a".repeat(100)));

        assert!(!channel_name(""));
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::AuthorNameTooLarge { chars: 257 }
        ));
    }

    #[test]
    fn test_embed_description_limit() {
        let mut embed = base_embed();
        embed.description.replace(str::repeat("a", 2048));
        assert!(super::embed(&embed).is_ok());

        embed.description.replace(str::repeat("a", 2049));
        assert!(matches!(
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::DescriptionTooLarge { chars: 2049 }
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::TooManyFields { amount: 26 }
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::FieldNameTooLarge { chars: 257 }
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::FieldValueTooLarge { chars: 1025 }
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::FooterTextTooLarge { chars: 2049 }
        ));
    }

    #[test]
    fn test_embed_title_limit() {
        let mut embed = base_embed();
        embed.title.replace(str::repeat("a", 256));
        assert!(super::embed(&embed).is_ok());

        embed.title.replace(str::repeat("a", 257));
        assert!(matches!(
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::TitleTooLarge { chars: 257 }
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
            super::embed(&embed).unwrap_err().kind(),
            EmbedValidationErrorType::EmbedTooLarge { chars: 6304 }
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
        assert!(guild_prune_days(30));
        assert!(!guild_prune_days(31));
        assert!(!guild_prune_days(100));
    }

    #[test]
    fn test_invite_max_age() {
        assert!(invite_max_age(0));
        assert!(invite_max_age(86_400));
        assert!(invite_max_age(604_800));
        assert!(!invite_max_age(604_801));
    }

    #[test]
    fn test_invite_max_uses() {
        assert!(invite_max_uses(0));
        assert!(invite_max_uses(100));
        assert!(!invite_max_uses(101));
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
