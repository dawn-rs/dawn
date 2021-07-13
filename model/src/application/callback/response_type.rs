use serde_repr::{Deserialize_repr, Serialize_repr};

/// Contains the possible response type integers for an interaction.
#[derive(
    Clone, Copy, Debug, Deserialize_repr, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize_repr,
)]
#[repr(u8)]
pub enum ResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    /// Ack an interaction and edit the original message later.
    ///
    /// This is only valid for components.
    DeferredUpdateMessage = 6,
    /// Edit the message a component is attached to.
    UpdateMessage = 7,
}

impl ResponseType {
    /// Name of the variant.
    ///
    /// The returned name is equivalent to the variant name.
    ///
    /// # Examples
    ///
    /// Check the names of the [`Pong`] and [`UpdateMessage`] variants:
    ///
    /// ```
    /// use twilight_model::application::callback::ResponseType;
    ///
    /// assert_eq!("Pong", ResponseType::Pong.name());
    /// assert_eq!("UpdateMessage", ResponseType::UpdateMessage.name());
    /// ```
    ///
    /// [`Pong`]: Self::Pong
    /// [`UpdateMessage`]: Self::UpdateMessage
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pong => "Pong",
            Self::ChannelMessageWithSource => "ChannelMessageWithSource",
            Self::DeferredChannelMessageWithSource => "DeferredChannelMessageWithSource",
            Self::DeferredUpdateMessage => "DeferredUpdateMessage",
            Self::UpdateMessage => "UpdateMessage",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResponseType;
    use serde::{Deserialize, Serialize};
    use static_assertions::{assert_impl_all, const_assert_eq};
    use std::{fmt::Debug, hash::Hash};

    assert_impl_all!(
        ResponseType: Clone,
        Copy,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
        Serialize,
        Send,
        Sync
    );
    const_assert_eq!(1, ResponseType::Pong as u8);
    const_assert_eq!(4, ResponseType::ChannelMessageWithSource as u8);
    const_assert_eq!(5, ResponseType::DeferredChannelMessageWithSource as u8);
    const_assert_eq!(6, ResponseType::DeferredUpdateMessage as u8);
    const_assert_eq!(7, ResponseType::UpdateMessage as u8);

    #[test]
    fn test_name() {
        assert_eq!("Pong", ResponseType::Pong.name());
        assert_eq!(
            "ChannelMessageWithSource",
            ResponseType::ChannelMessageWithSource.name()
        );
        assert_eq!(
            "DeferredChannelMessageWithSource",
            ResponseType::DeferredChannelMessageWithSource.name()
        );
        assert_eq!(
            "DeferredUpdateMessage",
            ResponseType::DeferredUpdateMessage.name()
        );
        assert_eq!("UpdateMessage", ResponseType::UpdateMessage.name());
    }
}
