use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

use super::ComponentType;
use crate::channel::ReactionType;

/// Dropdown-style interactive components that render on messages.
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#select-menus
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct SelectMenu {
    /// Developer defined identifier.
    pub custom_id: String,
    /// Maximum number of options that may be chosen.
    pub max_values: Option<u8>,
    /// Minimum number of options that must be chosen.
    pub min_values: Option<u8>,
    /// List of available choices.
    pub options: Vec<SelectMenuOption>,
    /// Custom placeholder text if no option is selected.
    pub placeholder: Option<String>,
}

/// Dropdown options that are part of [`SelectMenu`].
///
/// Refer to [Discord Docs/Message Components] for additional information.
///
/// [Discord Docs/Message Components]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SelectMenuOption {
    /// Whether the option will be selected by default.
    #[serde(default)]
    pub default: bool,
    /// Additional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
    /// User-facing name.
    pub label: String,
    /// Developer defined value.
    pub value: String,
}

impl Serialize for SelectMenu {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Base of 3 to account for the fields that are always present:
        //
        // - `custom_id`
        // - `options`
        // - `type`
        let field_count = 3
            + usize::from(self.placeholder.is_some())
            + usize::from(self.min_values.is_some())
            + usize::from(self.max_values.is_some());
        let mut state = serializer.serialize_struct("SelectMenu", field_count)?;

        state.serialize_field("custom_id", &self.custom_id)?;

        if self.max_values.is_some() {
            state.serialize_field("max_values", &self.max_values)?;
        }

        if self.min_values.is_some() {
            state.serialize_field("min_values", &self.min_values)?;
        }

        state.serialize_field("options", &self.options)?;

        if self.placeholder.is_some() {
            state.serialize_field("placeholder", &self.placeholder)?;
        }

        state.serialize_field("type", &ComponentType::SelectMenu)?;

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{SelectMenu, SelectMenuOption};
    use serde::{Deserialize, Serialize};
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(SelectMenuOption: default, description, emoji, label, value);
    assert_fields!(
        SelectMenu: custom_id,
        max_values,
        min_values,
        options,
        placeholder
    );
    assert_impl_all!(
        SelectMenuOption: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Send,
        Serialize,
        Sync
    );
    assert_impl_all!(
        SelectMenu: Clone,
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
