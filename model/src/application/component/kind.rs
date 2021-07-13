use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Type of Component.
///
/// Refer to [Discord Docs/Message Components] for more information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#component-types
#[derive(
    Clone, Copy, Debug, Deserialize_repr, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize_repr,
)]
#[repr(u8)]
pub enum ComponentType {
    /// Component is an [`ActionRow`].
    ///
    /// [`ActionRow`]: super::ActionRow
    ActionRow = 1,

    /// Component is an [`Button`].
    ///
    /// [`Button`]: super::Button
    Button = 2,

    /// Component is an [`SelectMenu`].
    ///
    /// [`SelectMenu`]: super::SelectMenu
    SelectMenu = 3,
}

impl ComponentType {
    /// Name of the component type.
    ///
    /// Variants have a name equivalent to the variant name itself.
    ///
    /// # Examples
    ///
    /// Check the [`ActionRow`] variant's name:
    ///
    /// ```
    /// use twilight_model::application::component::ComponentType;
    ///
    /// assert_eq!("ActionRow", ComponentType::ActionRow.name());
    /// ```
    ///
    /// [`ActionRow`]: Self::ActionRow
    pub const fn name(self) -> &'static str {
        match self {
            Self::ActionRow => "ActionRow",
            Self::Button => "Button",
            Self::SelectMenu => "SelectMenu",
        }
    }
}

impl Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentType;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_impl_all, const_assert_eq};
    use std::{fmt::Debug, hash::Hash};

    assert_impl_all!(
        ComponentType: Clone,
        Copy,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
        Send,
        Serialize,
        Sync
    );
    const_assert_eq!(1, ComponentType::ActionRow as u8);
    const_assert_eq!(2, ComponentType::Button as u8);
    const_assert_eq!(3, ComponentType::SelectMenu as u8);

    #[test]
    fn test_variants() {
        serde_test::assert_tokens(&ComponentType::ActionRow, &[Token::U8(1)]);
        serde_test::assert_tokens(&ComponentType::Button, &[Token::U8(2)]);
        serde_test::assert_tokens(&ComponentType::SelectMenu, &[Token::U8(3)]);
    }

    #[test]
    fn test_names() {
        assert_eq!("ActionRow", ComponentType::ActionRow.name());
        assert_eq!("Button", ComponentType::Button.name());
        assert_eq!("SelectMenu", ComponentType::SelectMenu.name());
    }
}
