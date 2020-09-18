use super::ParseMention;
use std::{iter::Iterator, marker::PhantomData, str::CharIndices};

/// Iterator of mentions within a buffer.
///
/// Unlike when parsing a mention directly via [`ParseMention::parse`],
/// incomplete mentions - such as when a "<" is found but not valid trailing
/// mention syntax - will not result in errors.
///
/// The iterator returns items consisting of 3 items: the mention itself
/// followed by the starting index and ending index of the mention's source in
/// the buffer.
///
/// # Examples
///
/// Iterate over all of the mentioned users:
///
/// ```
/// use twilight_mention::ParseMention;
/// use twilight_model::id::UserId;
///
/// let buf = "<@123> some <@456> users <@789>!";
/// let mut iter = UserId::iter(buf);
/// assert!(matches!(iter.next(), Some((UserId(123), _, _))));
/// assert!(matches!(iter.next(), Some((UserId(456), _, _))));
/// assert!(matches!(iter.next(), Some((UserId(789), _, _))));
/// ```
#[derive(Clone, Debug)]
pub struct MentionIter<'a, T> {
    buf: &'a str,
    chars: CharIndices<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> MentionIter<'a, T> {
    #[must_use]
    pub(in crate::parse) fn new(buf: &'a str) -> Self {
        let chars = buf.char_indices();

        Self {
            buf,
            chars,
            phantom: PhantomData,
        }
    }

    /// Return an immutable reference to the underlying buffer of the iterator.
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.buf
    }
}

impl<'a, T: ParseMention + std::fmt::Debug> Iterator for MentionIter<'a, T> {
    /// Found mention followed by the start and ending indexes in the source
    /// string returned by [`as_str`].
    ///
    /// [`as_str`]: #method.as_str
    type Item = (T, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // We want to take care of our input and make sure we're working with
        // chars here and not just individual bytes. We also want to not use
        // consuming methods of the iterator, so this will get a little weird.
        'outer: loop {
            let start = match self.chars.next()? {
                (idx, '<') => idx,
                _ => continue,
            };

            for ch in T::SIGIL.chars() {
                if self.chars.next()?.1 != ch {
                    continue 'outer;
                }
            }

            let end = match self.chars.find(|c| c.1 == '>') {
                Some((idx, _)) => idx,
                None => continue,
            };
            let buf = self.buf.get(start..=end)?;

            if let Ok(id) = T::parse(buf) {
                return Some((id, start, end));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ParseMention, MentionIter};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;
    use twilight_model::id::{ChannelId, EmojiId, RoleId, UserId};

    assert_impl_all!(MentionIter<'_, ChannelId>: Clone, Debug, Iterator, Send, Sync);
    assert_impl_all!(MentionIter<'_, EmojiId>: Clone, Debug, Iterator, Send, Sync);
    assert_impl_all!(MentionIter<'_, RoleId>: Clone, Debug, Iterator, Send, Sync);
    assert_impl_all!(MentionIter<'_, UserId>: Clone, Debug, Iterator, Send, Sync);
    assert_obj_safe!(
        MentionIter<'_, ChannelId>,
        MentionIter<'_, EmojiId>,
        MentionIter<'_, RoleId>,
        MentionIter<'_, UserId>,
    );

    #[test]
    fn test_iter_channel_id() {
        let mut iter = ChannelId::iter("<#123>");
        assert_eq!(ChannelId(123), iter.next().unwrap().0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_multiple_ids() {
        let buf = "one <@123>two<#456><@789> ----";
        let mut iter = UserId::iter(buf);
        assert_eq!(UserId(123), iter.next().unwrap().0);
        let (mention, start, end) = iter.next().unwrap();
        assert_eq!(UserId(789), mention);
        assert_eq!(19, start);
        assert_eq!(24, end);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_emoji_ids() {
        let mut iter = EmojiId::iter("some <:name:123> emojis <:emoji:456>");
        assert_eq!(EmojiId(123), iter.next().unwrap().0);
        assert_eq!(EmojiId(456), iter.next().unwrap().0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_role_ids() {
        let mut iter = RoleId::iter("some <@&123> roles <@&456>");
        assert_eq!(RoleId(123), iter.next().unwrap().0);
        assert_eq!(RoleId(456), iter.next().unwrap().0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_user_ids() {
        let mut iter = UserId::iter("some <@123>users<@456>");
        assert_eq!(UserId(123), iter.next().unwrap().0);
        assert_eq!(UserId(456), iter.next().unwrap().0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_no_id() {
        let mention = "this is not <# actually a mention";
        let mut iter = ChannelId::iter(mention);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_ignores_other_types() {
        let mention = "<#123> <:name:456> <@&789>";
        let mut iter = UserId::iter(mention);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_as_str() {
        let buf = "a buf";
        let mut iter = RoleId::iter(buf);
        assert_eq!(buf, iter.as_str());
        // Advancing still returns the complete buffer.
        assert!(iter.next().is_none());
        assert_eq!(buf, iter.as_str());
    }
}
