pub mod get_reactions;

pub(crate) mod delete_reaction;

mod create_reaction;
mod delete_all_reaction;
mod delete_all_reactions;

pub use self::{
    create_reaction::CreateReaction, delete_all_reaction::DeleteAllReaction,
    delete_all_reactions::DeleteAllReactions, delete_reaction::DeleteReaction,
    get_reactions::GetReactions,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twilight_model::id::EmojiId;

/// Handle a reaction of either a custom or unicode emoji.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RequestReactionType<'a> {
    /// Reaction of a custom emoji.
    Custom {
        /// ID of the custom emoji.
        id: EmojiId,
        /// Name of the custom emoji.
        ///
        /// This is not strictly required, but may be helpful for Discord to
        /// work with.
        name: Option<&'a str>,
    },
    /// Reaction of a unicode emoji, such as "🌈".
    Unicode {
        /// Unicode emoji.
        name: &'a str,
    },
}

impl<'a> RequestReactionType<'a> {
    /// Create a display formatter for a reaction type resulting in a format
    /// acceptable for use in URLs.
    ///
    /// # Examples
    ///
    /// Format the transgender flag for use in a URL:
    ///
    /// ```
    /// use twilight_http::request::channel::reaction::RequestReactionType;
    /// use twilight_model::id::EmojiId;
    ///
    /// let reaction = RequestReactionType::Unicode {
    ///     name: "🏳️‍⚧️",
    /// };
    ///
    /// // Retrieve the display formatter.
    /// let display = reaction.display();
    ///
    /// // And now format it into a percent-encoded string and then check it.
    /// assert_eq!(
    ///     "%F0%9F%8F%B3%EF%B8%8F%E2%80%8D%E2%9A%A7%EF%B8%8F",
    ///     display.to_string(),
    /// );
    /// ```
    pub const fn display(&'a self) -> RequestReactionTypeDisplay<'a> {
        RequestReactionTypeDisplay(self)
    }
}

/// Format a [`RequestReactionType`] into a format acceptable for use in URLs.
///
/// # Examples
///
/// Format a custom reaction for use in a URL:
///
/// ```
/// use twilight_http::request::channel::reaction::RequestReactionType;
/// use twilight_model::id::EmojiId;
///
/// let reaction = RequestReactionType::Custom {
///     id: EmojiId(123),
///     name: Some("rarity"),
/// };
///
/// // Retrieve the display formatter.
/// let display = reaction.display();
///
/// // And now format it into an acceptable string and then check it.
/// assert_eq!("rarity:123", display.to_string());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestReactionTypeDisplay<'a>(&'a RequestReactionType<'a>);

impl Display for RequestReactionTypeDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.0 {
            RequestReactionType::Custom { id, name } => {
                if let Some(name) = name {
                    f.write_str(name)?;
                } else {
                    f.write_str("e")?;
                }

                f.write_str(":")?;

                Display::fmt(id, f)
            }
            RequestReactionType::Unicode { name } => {
                Display::fmt(&utf8_percent_encode(&name, NON_ALPHANUMERIC), f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // `clippy::non_ascii_literal` can't be allowed on an item level; it can
    // only be enabled on a module level.
    #![allow(clippy::non_ascii_literal)]

    use super::{RequestReactionType, RequestReactionTypeDisplay};
    use static_assertions::{assert_fields, assert_impl_all};
    use std::fmt::{Debug, Display};
    use twilight_model::id::EmojiId;

    assert_fields!(RequestReactionType::Custom: id, name);
    assert_fields!(RequestReactionType::Unicode: name);
    assert_impl_all!(RequestReactionTypeDisplay<'_>: Clone, Debug, Display, Eq, PartialEq, Send, Sync);
    assert_impl_all!(RequestReactionType<'_>: Clone, Debug, Eq, PartialEq, Send, Sync);

    #[test]
    fn test_display_custom_with_name() {
        let reaction = RequestReactionType::Custom {
            id: EmojiId(123),
            name: Some("foo"),
        };

        assert_eq!("foo:123", reaction.display().to_string());
    }

    #[test]
    fn test_display_custom_without_name() {
        let reaction = RequestReactionType::Custom {
            id: EmojiId(123),
            name: None,
        };

        assert_eq!("e:123", reaction.display().to_string());
    }

    /// Test that unicode reactions format with percent encoding.
    // We can't use the actual flag here
    #[test]
    fn test_display_unicode() {
        let reaction = RequestReactionType::Unicode {
            // Rainbow flag 🏳️‍🌈
            name: "🏳️‍🌈",
        };

        assert_eq!(
            "%F0%9F%8F%B3%EF%B8%8F%E2%80%8D%F0%9F%8C%88",
            reaction.display().to_string()
        );
    }
}
