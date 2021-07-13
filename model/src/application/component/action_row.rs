use super::{Component, ComponentType};
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

/// A non-interactive component that acts as a container for other components.
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#action-rows
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct ActionRow {
    /// List of components in the action row.
    pub components: Vec<Component>,
}

impl Serialize for ActionRow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let field_count = 1 + usize::from(!self.components.is_empty());
        let mut state = serializer.serialize_struct("ActionRow", field_count)?;

        if !self.components.is_empty() {
            state.serialize_field("components", &self.components)?;
        }

        state.serialize_field("type", &ComponentType::ActionRow)?;

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::ActionRow;
    use serde::{Deserialize, Serialize};
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(ActionRow: components);
    assert_impl_all!(
        ActionRow: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Send,
        Serialize,
        Sync
    );
}
