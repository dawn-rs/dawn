use serde::Serialize;
use twilight_model::{
    guild::Emoji,
    id::{EmojiId, RoleId, UserId},
};

/// Represents a cached [`Emoji`].
///
/// [`Emoji`]: twilight_model::guild::Emoji
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CachedEmoji {
    /// Id of the Emoji.
    pub id: EmojiId,
    /// Whether the emoji is animated.
    pub animated: bool,
    /// Name of the Emoji.
    pub name: String,
    /// Whether this emoji can be used.
    ///
    /// May be false due to loss of Server Boosts.
    pub available: bool,
    /// Whether the emoji is managed.
    pub managed: bool,
    /// Whether the emoji must be wrapped in colons.
    pub require_colons: bool,
    /// List of roles allowed to use this emoji.
    pub roles: Vec<RoleId>,
    /// ID of the user who created the emoji.
    pub user_id: Option<UserId>,
}

impl PartialEq<Emoji> for CachedEmoji {
    fn eq(&self, other: &Emoji) -> bool {
        self.id == other.id
            && self.animated == other.animated
            && self.managed == other.managed
            && self.name == other.name
            && self.require_colons == other.require_colons
            && self.roles == other.roles
            && self.user_id == other.user.as_ref().map(|user| user.id)
            && self.available == other.available
    }
}

#[cfg(test)]
mod tests {
    use super::CachedEmoji;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::fmt::Debug;
    use twilight_model::{guild::Emoji, id::EmojiId};

    assert_fields!(
        CachedEmoji: id,
        animated,
        managed,
        name,
        require_colons,
        roles,
        user_id
    );
    assert_impl_all!(CachedEmoji: Clone, Debug, Eq, PartialEq);

    #[test]
    fn test_eq_emoji() {
        let emoji = Emoji {
            id: EmojiId(123),
            animated: true,
            name: "foo".to_owned(),
            available: true,
            managed: false,
            require_colons: true,
            roles: vec![],
            user: None,
        };
        let cached = CachedEmoji {
            id: EmojiId(123),
            animated: true,
            available: true,
            name: "foo".to_owned(),
            managed: false,
            require_colons: true,
            roles: vec![],
            user_id: None,
        };

        assert_eq!(cached, emoji);
    }
}
