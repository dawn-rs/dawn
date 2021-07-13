/// Contains all of the input validation functions for requests.
///
/// This is in a centralised place so that the validation parameters can be kept
/// up-to-date more easily and because some of the checks are re-used across
/// different modules.
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    application::component::{select_menu::SelectMenuOption, Component, ComponentType},
    channel::embed::Embed,
};

/// A provided [`Component`] is invalid.
///
/// While multiple components may be invalid, validation will short-circuit on
/// the first invalid component.
#[derive(Debug)]
pub struct ComponentValidationError {
    kind: ComponentValidationErrorType,
}

impl ComponentValidationError {
    /// Maximum number of [`Component`]s allowed inside an [`ActionRow`].
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Action Rows][1].
    ///
    /// [`ActionRow`]: twilight_model::application::component::ActionRow
    /// [1]: https://discord.com/developers/docs/interactions/message-components#action-rows
    pub const ACTION_ROW_COMPONENT_COUNT: usize = 5;

    /// Maximum number of root [`Component`]s in a message.
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Action Row][1].
    ///
    /// [1]: https://discord.com/developers/docs/interactions/message-components#action-rows
    pub const COMPONENT_COUNT: usize = 5;

    /// Maximum length of a [`Component`] custom ID in codepoints.
    ///
    /// An example of a component with a custom ID is the
    /// [`Button`][`Button::custom_id`].
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Components][1].
    ///
    /// [`Button::custom_id`]: twilight_model::application::component::button::Button::custom_id
    /// [1]: https://discord.com/developers/docs/interactions/message-components#component-object-component-structure
    pub const COMPONENT_CUSTOM_ID_LENGTH: usize = 100;

    /// Maximum [`Component`] label length in codepoints.
    ///
    /// An example of a component with a label is the [`Button`][`Button::label`].
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Components][1].
    ///
    /// [`Button::label`]: twilight_model::application::component::button::Button::label
    /// [1]: https://discord.com/developers/docs/interactions/message-components#component-object-component-structure
    pub const COMPONENT_LABEL_LENGTH: usize = 80;

    /// Maximum number of [`SelectMenuOption`]s that can be chosen in a
    /// [`SelectMenu`].
    ///
    /// This is defined in Dicsord's documentation, per
    /// [Discord Docs/Select Menu][1].
    ///
    /// [`SelectMenuOption`]: twilight_model::application::component::select_menu::SelectMenuOption
    /// [`SelectMenu`]: twilight_model::application::component::select_menu::SelectMenu
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure
    pub const SELECT_MAXIMUM_VALUES_LIMIT: usize = 25;

    /// Minimum number of [`SelectMenuOption`]s that can be chosen in a
    /// [`SelectMenu`].
    ///
    /// This is defined in Dicsord's documentation, per
    /// [Discord Docs/Select Menu][1].
    ///
    /// [`SelectMenuOption`]: twilight_model::application::component::select_menu::SelectMenuOption
    /// [`SelectMenu`]: twilight_model::application::component::select_menu::SelectMenu
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure
    pub const SELECT_MAXIMUM_VALUES_REQUIREMENT: usize = 1;

    /// Maximum number of [`SelectMenuOption`]s that must be chosen in a
    /// [`SelectMenu`].
    ///
    /// This is defined in Dicsord's documentation, per
    /// [Discord Docs/Select Menu][1].
    ///
    /// [`SelectMenuOption`]: twilight_model::application::component::select_menu::SelectMenuOption
    /// [`SelectMenu`]: twilight_model::application::component::select_menu::SelectMenu
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure
    pub const SELECT_MINIMUM_VALUES_LIMIT: usize = 25;

    /// Maximum number of [`SelectMenuOption`]s in a [`SelectMenu`].
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Select Menu][1].
    ///
    /// [`SelectMenuOption`]: twilight_model::application::component::select_menu::SelectMenuOption
    /// [`SelectMenu`]: twilight_model::application::component::select_menu::SelectMenu
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure
    pub const SELECT_OPTION_COUNT: usize = 25;

    /// Maximum length of a [`SelectMenuOption::description`] in codepoints.
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Select Menu Option][1].
    ///
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure
    pub const SELECT_OPTION_DESCRIPTION_LENGTH: usize = 50;

    /// Maximum length of a [`SelectMenuOption::label`] in codepoints.
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Select Menu Option][1].
    ///
    /// [`SelectMenuOption::label`]: twilight_model::application::component::select_menu::SelectMenuOption::label
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure
    pub const SELECT_OPTION_LABEL_LENGTH: usize = 25;

    /// Maximum length of a [`SelectMenuOption::value`] in codepoints.
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Select Menu Option][1].
    ///
    /// [`SelectMenuOption::value`]: twilight_model::application::component::select_menu::SelectMenuOption::value
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-option-structure
    pub const SELECT_OPTION_VALUE_LENGTH: usize = 100;

    /// Maximum length of a [`SelectMenu::placeholder`] in codepoints.
    ///
    /// This is defined in Discord's documentation, per
    /// [Discord Docs/Select Menu][1].
    ///
    /// [`SelectMenu::placeholder`]: twilight_model::application::component::select_menu::SelectMenu::placeholder
    /// [1]: https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure
    pub const SELECT_PLACEHOLDER_LENGTH: usize = 100;

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        ComponentValidationErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for ComponentValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ComponentValidationErrorType::ActionRowComponentCount { count } => {
                f.write_str("an action row has ")?;
                Display::fmt(&count, f)?;
                f.write_str(" children, but the max is ")?;

                Display::fmt(&Self::ACTION_ROW_COMPONENT_COUNT, f)
            }
            ComponentValidationErrorType::ComponentCount { count } => {
                Display::fmt(count, f)?;
                f.write_str(" components were provided, but the max is ")?;

                Display::fmt(&Self::COMPONENT_COUNT, f)
            }
            ComponentValidationErrorType::ComponentCustomIdLength { chars } => {
                f.write_str("a component's custom id is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::COMPONENT_CUSTOM_ID_LENGTH, f)
            }
            ComponentValidationErrorType::ComponentLabelLength { chars } => {
                f.write_str("a component's label is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::COMPONENT_LABEL_LENGTH, f)
            }
            ComponentValidationErrorType::InvalidChildComponent { kind } => {
                f.write_str("a '")?;
                Display::fmt(&kind, f)?;

                f.write_str(" component was provided, but can not be a child component")
            }
            ComponentValidationErrorType::InvalidRootComponent { kind } => {
                f.write_str("a '")?;
                Display::fmt(kind, f)?;

                f.write_str("' component was provided, but can not be a root component")
            }
            ComponentValidationErrorType::SelectMaximumValuesCount { count } => {
                f.write_str("maximum number of values that can be chosen is ")?;
                Display::fmt(count, f)?;
                f.write_str(", but must be greater than or equal to ")?;
                Display::fmt(&Self::SELECT_MAXIMUM_VALUES_REQUIREMENT, f)?;
                f.write_str("and less than or equal to ")?;

                Display::fmt(&Self::SELECT_MAXIMUM_VALUES_LIMIT, f)
            }
            ComponentValidationErrorType::SelectMinimumValuesCount { count } => {
                f.write_str("maximum number of values that must be chosen is ")?;
                Display::fmt(count, f)?;
                f.write_str(", but must be less than or equal to ")?;

                Display::fmt(&Self::SELECT_MAXIMUM_VALUES_LIMIT, f)
            }
            ComponentValidationErrorType::SelectOptionDescriptionLength { chars } => {
                f.write_str("a select menu option's description is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::SELECT_OPTION_DESCRIPTION_LENGTH, f)
            }
            ComponentValidationErrorType::SelectOptionLabelLength { chars } => {
                f.write_str("a select menu option's label is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::SELECT_OPTION_LABEL_LENGTH, f)
            }
            ComponentValidationErrorType::SelectOptionValueLength { chars } => {
                f.write_str("a select menu option's value is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::SELECT_OPTION_VALUE_LENGTH, f)
            }
            ComponentValidationErrorType::SelectPlaceholderLength { chars } => {
                f.write_str("a select menu's placeholder is ")?;
                Display::fmt(&chars, f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::SELECT_PLACEHOLDER_LENGTH, f)
            }
            ComponentValidationErrorType::SelectOptionCount { count } => {
                f.write_str("a select menu has ")?;
                Display::fmt(&count, f)?;
                f.write_str(" options, but the max is ")?;

                Display::fmt(&Self::SELECT_OPTION_COUNT, f)
            }
        }
    }
}

impl Error for ComponentValidationError {}

/// Type of [`ComponentValidationError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ComponentValidationErrorType {
    /// Number of components a provided [`ActionRow`] is larger than
    /// [the maximum][`ACTION_ROW_COMPONENT_COUNT`].
    ///
    /// [`ActionRow`]: twilight_model::application::component::ActionRow
    /// [`ACTION_ROW_COMPONENT_COUNT`]: ComponentValidationError::ACTION_ROW_COMPONENT_COUNT
    ActionRowComponentCount {
        /// Number of components within the action row.
        count: usize,
    },
    /// Number of components provided is larger than
    /// [the maximum][`COMPONENT_COUNT`].
    ///
    /// [`COMPONENT_COUNT`]: ComponentValidationError::COMPONENT_COUNT
    ComponentCount {
        /// Number of components that were provided.
        count: usize,
    },
    /// Component custom ID is larger than the
    /// [the maximum][`COMPONENT_CUSTOM_ID_LENGTH`].
    ///
    /// [`COMPONENT_CUSTOM_ID_LENGTH`]: ComponentValidationError::COMPONENT_CUSTOM_ID_LENGTH
    ComponentCustomIdLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
    /// Component label is larger than [the maximum][`COMPONENT_LABEL_LENGTH`].
    ///
    /// [`COMPONENT_LABEL_LENGTH`]: ComponentValidationError::COMPONENT_LABEL_LENGTH
    ComponentLabelLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
    /// Provided component cannot be a child component.
    InvalidChildComponent {
        /// Type of provided component.
        kind: ComponentType,
    },
    /// Provided component cannot be a root component.
    InvalidRootComponent {
        /// Type of provided component.
        kind: ComponentType,
    },
    /// Maximum number of items that can be chosen is smaller than
    /// [the minimum][`SELECT_MAXIMUM_VALUES_REQUIREMENT`] or larger than
    /// [the maximum][`SELECT_MAXIMUM_VALUES_LIMIT`].
    ///
    /// [`SELECT_MAXIMUM_VALUES_LIMIT`]: ComponentValidationError::SELECT_MAXIMUM_VALUES_LIMIT
    /// [`SELECT_MAXIMUM_VALUES_REQUIREMENT`]: ComponentValidationError::SELECT_MAXIMUM_VALUES_REQUIREMENT
    SelectMaximumValuesCount { count: usize },
    /// Minimum number of items that must be chosen is larger than
    /// [the maximum][`SELECT_MINIMUM_VALUES_LIMIT`].
    ///
    /// [`SELECT_MINIMUM_VALUES_LIMIT`]: ComponentValidationError::SELECT_MINIMUM_VALUES_LIMIT
    SelectMinimumValuesCount { count: usize },
    /// Number of select menu options provided is larger than
    /// [the maximum][`SELECT_OPTION_COUNT`].
    ///
    /// [`SELECT_OPTION_COUNT`]: ComponentValidationError::SELECT_OPTION_COUNT
    SelectOptionCount {
        /// Number of options that were provided.
        count: usize,
    },
    /// Description of a select menu option is larger than
    /// [the maximum][`SELECT_OPTION_DESCRIPTION_LENGTH`].
    ///
    /// [`SELECT_OPTION_DESCRIPTION_LENGTH`]: ComponentValidationError::SELECT_OPTION_DESCRIPTION_LENGTH
    SelectOptionDescriptionLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
    /// Label of a select menu option is larger than
    /// [the maximum][`SELECT_OPTION_LABEL_LENGTH`].
    ///
    /// [`SELECT_OPTION_LABEL_LENGTH`]: ComponentValidationError::SELECT_OPTION_LABEL_LENGTH
    SelectOptionLabelLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
    /// Value of a select menu option is larger than
    /// [the maximum][`SELECT_OPTION_VALUE_LENGTH`].
    ///
    /// [`SELECT_OPTION_VALUE_LENGTH`]: ComponentValidationError::SELECT_OPTION_VALUE_LENGTH
    SelectOptionValueLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
    /// Placeholder of a component is larger than the
    /// [maximum][`SELECT_PLACEHOLDER_LENGTH`].
    ///
    /// [`SELECT_PLACEHOLDER_LENGTH`]: ComponentValidationError::SELECT_PLACEHOLDER_LENGTH
    SelectPlaceholderLength {
        /// Number of codepoints that were provided.
        chars: usize,
    },
}

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

/// Validate a list of components.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::ComponentCount`] if there are
/// too many components in the provided list.
///
/// Refer to the errors section of [`component`] for a list of errors that may
/// be returned as a result of validating each provided component.
pub fn components(components: &[Component]) -> Result<(), ComponentValidationError> {
    let count = components.len();

    if count > ComponentValidationError::COMPONENT_COUNT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::ComponentCount { count },
        });
    }

    for component in components {
        self::component(component)?;
    }

    Ok(())
}

/// Validate the contents of a component.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::ActionRowComponentCount`] error
/// type if the provided list of components is too many for an [`ActionRow`].
///
/// Returns a [`ComponentValidationErrorType::InvalidChildComponent`] if the
/// provided nested component is an [`ActionRow`]. Action rows can not
/// contain another action row.
///
/// [`ActionRow`]: twilight_model::application::component::ActionRow
pub fn component(component: &Component) -> Result<(), ComponentValidationError> {
    match component {
        Component::ActionRow(action_row) => {
            component_action_row_components(&action_row.components)?;

            for inner in &action_row.components {
                self::component_inner(inner)?;
            }
        }
        other => {
            return Err(ComponentValidationError {
                kind: ComponentValidationErrorType::InvalidRootComponent { kind: other.kind() },
            });
        }
    }

    Ok(())
}

/// Validate the contents of a component that is within another component, i.e.
/// one that is not a root component.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::InvalidChildComponent`] if a
/// provided nested component is a [`Component::ActionRow`]. Action rows can not
/// contain another action row.
///
/// Returns a [`ComponentValidationErrorType::OptionDescriptionLength`] error
/// type if a provided select option description is too long.
///
/// Returns a [`ComponentValidationErrorType::OptionLabelLength`] error type if
/// a provided select option label is too long.
///
/// Returns a [`ComponentValidationErrorType::OptionValueLength`] error type if
/// a provided select option value is too long.
///
/// Returns a [`ComponentValidationErrorType::SelectMaximumValuesCount`] if the
/// provided number of select menu values that can be chosen is smaller than the minimum or
/// larger than the maximum.
///
/// Returns a [`ComponentValidationErrorType::SelectMinimumValuesCount`] if the
/// provided number of select menu values that must be chosen is larger than the
/// maximum.
///
/// Returns a [`ComponentValidationErrorType::SelectPlaceholderLength`] error type if
/// a provided select placeholder is too long.
fn component_inner(component: &Component) -> Result<(), ComponentValidationError> {
    match component {
        Component::ActionRow(_) => {
            return Err(ComponentValidationError {
                kind: ComponentValidationErrorType::InvalidChildComponent {
                    kind: ComponentType::ActionRow,
                },
            })
        }
        Component::Button(button) => {
            if let Some(custom_id) = button.custom_id.as_ref() {
                component_custom_id(custom_id)?;
            }

            if let Some(label) = button.label.as_ref() {
                component_label(label)?;
            }
        }
        Component::SelectMenu(select_menu) => {
            component_custom_id(&select_menu.custom_id)?;
            component_select_options(&select_menu.options)?;

            if let Some(placeholder) = select_menu.placeholder.as_ref() {
                component_select_placeholder(placeholder)?;
            }

            if let Some(max_values) = select_menu.max_values {
                component_select_max_values(usize::from(max_values))?;
            }

            if let Some(min_values) = select_menu.min_values {
                component_select_min_values(usize::from(min_values))?;
            }

            for option in &select_menu.options {
                component_select_option_label(&option.label)?;
                component_select_option_value(&option.value)?;

                if let Some(description) = option.description.as_ref() {
                    component_option_description(description)?;
                }
            }
        }
    }

    Ok(())
}

/// Validate that an [`ActionRow`] does not contain too many components.
///
/// [`ActionRow`]s may only have so many components within it, defined by
/// [`ComponentValidationError::ACTION_ROW_COMPONENT_COUNT`].
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::ActionRowComponentCount`] error
/// type if the provided list of components is too many for an [`ActionRow`].
const fn component_action_row_components(
    components: &[Component],
) -> Result<(), ComponentValidationError> {
    let count = components.len();

    if count > ComponentValidationError::COMPONENT_COUNT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::ActionRowComponentCount { count },
        });
    }

    Ok(())
}

/// Validate that a [`Component`]s label is not too long.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::ComponentLabelLength`] if the
/// provided component label is too long.
fn component_label(label: &str) -> Result<(), ComponentValidationError> {
    let chars = label.chars().count();

    if chars > ComponentValidationError::COMPONENT_LABEL_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::ComponentLabelLength { chars },
        });
    }

    Ok(())
}

/// Validate that a custom ID is not too long.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::ComponentCustomIdLength`] if the provided
/// custom ID is too long.
fn component_custom_id(custom_id: &str) -> Result<(), ComponentValidationError> {
    let chars = custom_id.chars().count();

    if chars > ComponentValidationError::COMPONENT_CUSTOM_ID_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::ComponentCustomIdLength { chars },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenuOption::description`]'s length.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectOptionDescriptionLength`] if the
/// provided select option description is too long.
///
/// [`SelectMenuOption::description`]: twilight_model::application::component::select_menu::SelectMenuOption::description
fn component_option_description(description: &str) -> Result<(), ComponentValidationError> {
    let chars = description.chars().count();

    if chars > ComponentValidationError::SELECT_OPTION_DESCRIPTION_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectOptionDescriptionLength { chars },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenu::max_values`] amount.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectMaximumValuesCount`] if the
/// provided number of values that can be chosen is smaller than
/// [the minimum][`SELECT_MAXIMUM_VALUES_REQUIREMENT`] or larger than
/// [the maximum][`SELECT_MAXIMUM_VALUES_LIMIT`].
///
/// [`SELECT_MAXIMUM_VALUES_LIMIT`]: ComponentValidationError::SELECT_MAXIMUM_VALUES_LIMIT
/// [`SELECT_MAXIMUM_VALUES_REQUIREMENT`]: ComponentValidationError::SELECT_MAXIMUM_VALUES_REQUIREMENT
/// [`SelectMenu::max_values`]: twilight_model::application::component::select_menu::SelectMenu::max_values
const fn component_select_max_values(count: usize) -> Result<(), ComponentValidationError> {
    if count > ComponentValidationError::SELECT_MAXIMUM_VALUES_LIMIT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectMaximumValuesCount { count },
        });
    }

    if count < ComponentValidationError::SELECT_MAXIMUM_VALUES_REQUIREMENT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectMaximumValuesCount { count },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenu::min_values`] amount.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectMinimumValuesCount`] if the
/// provided number of values that must be chosen is larger than
/// [the maximum][`SELECT_MINIMUM_VALUES_LIMIT`].
///
/// [`SELECT_MINIMUM_VALUES_LIMIT`]: ComponentValidationError::SELECT_MINIMUM_VALUES_LIMIT
/// [`SelectMenu::min_values`]: twilight_model::application::component::select_menu::SelectMenu::min_values
const fn component_select_min_values(count: usize) -> Result<(), ComponentValidationError> {
    if count > ComponentValidationError::SELECT_MINIMUM_VALUES_LIMIT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectMinimumValuesCount { count },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenuOption::label`]'s length.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectOptionLabelLength`] if the
/// provided select option label is too long.
///
/// [`SelectMenuOption::label`]: twilight_model::application::component::select_menu::SelectMenuOption::label
fn component_select_option_label(label: &str) -> Result<(), ComponentValidationError> {
    let chars = label.chars().count();

    if chars > ComponentValidationError::SELECT_OPTION_LABEL_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectOptionLabelLength { chars },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenuOption::value`]'s length.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectOptionValueLength`] if the
/// provided select option value is too long.
///
/// [`SelectMenuOption::value`]: twilight_model::application::component::select_menu::SelectMenuOption::value
fn component_select_option_value(value: &str) -> Result<(), ComponentValidationError> {
    let chars = value.chars().count();

    if chars > ComponentValidationError::SELECT_OPTION_VALUE_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectOptionValueLength { chars },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenu`]s number of [`options`].
///
/// [`Component::SelectMenu`]s may only have so many options within it, defined
/// by [`ComponentValidationError::SELECT_OPTION_COUNT`].
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectOptionCount`] error type if
/// the provided list of [`SelectMenuOption`]s is too many for a [`SelectMenu`].
///
/// [`SelectMenuOption`]: twilight_model::application::component::select_menu::SelectMenuOption
/// [`SelectMenu::options`]: twilight_model::application::component::select_menu::SelectMenu::options
/// [`SelectMenu`]: twilight_model::application::component::select_menu::SelectMenu
const fn component_select_options(
    options: &[SelectMenuOption],
) -> Result<(), ComponentValidationError> {
    let count = options.len();

    if count > ComponentValidationError::SELECT_OPTION_COUNT {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectOptionCount { count },
        });
    }

    Ok(())
}

/// Validate a [`SelectMenu::placeholder`]'s length.
///
/// # Errors
///
/// Returns a [`ComponentValidationErrorType::SelectPlaceholderLength`] if the
/// provided select placeholder is too long.
///
/// [`SelectMenu::placeholder`]: twilight_model::application::component::select_menu::SelectMenu::placeholder
fn component_select_placeholder(placeholder: &str) -> Result<(), ComponentValidationError> {
    let chars = placeholder.chars().count();

    if chars > ComponentValidationError::SELECT_PLACEHOLDER_LENGTH {
        return Err(ComponentValidationError {
            kind: ComponentValidationErrorType::SelectPlaceholderLength { chars },
        });
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::{assert_fields, assert_impl_all, const_assert_eq};
    use std::fmt::Debug;
    use twilight_model::channel::embed::{EmbedAuthor, EmbedField, EmbedFooter};

    assert_fields!(ComponentValidationErrorType::ActionRowComponentCount: count);
    assert_fields!(ComponentValidationErrorType::ComponentCount: count);
    assert_fields!(ComponentValidationErrorType::ComponentCustomIdLength: chars);
    assert_fields!(ComponentValidationErrorType::ComponentLabelLength: chars);
    assert_fields!(ComponentValidationErrorType::InvalidChildComponent: kind);
    assert_fields!(ComponentValidationErrorType::InvalidRootComponent: kind);
    assert_fields!(ComponentValidationErrorType::SelectMaximumValuesCount: count);
    assert_fields!(ComponentValidationErrorType::SelectMinimumValuesCount: count);
    assert_fields!(ComponentValidationErrorType::SelectOptionDescriptionLength: chars);
    assert_fields!(ComponentValidationErrorType::SelectOptionLabelLength: chars);
    assert_fields!(ComponentValidationErrorType::SelectOptionValueLength: chars);
    assert_fields!(ComponentValidationErrorType::SelectPlaceholderLength: chars);
    assert_impl_all!(ComponentValidationErrorType: Debug, Send, Sync);
    assert_impl_all!(ComponentValidationError: Debug, Send, Sync);
    assert_impl_all!(EmbedValidationErrorType: Debug, Send, Sync);
    assert_impl_all!(EmbedValidationError: Debug, Send, Sync);
    const_assert_eq!(5, ComponentValidationError::ACTION_ROW_COMPONENT_COUNT);
    const_assert_eq!(5, ComponentValidationError::COMPONENT_COUNT);
    const_assert_eq!(100, ComponentValidationError::COMPONENT_CUSTOM_ID_LENGTH);
    const_assert_eq!(80, ComponentValidationError::COMPONENT_LABEL_LENGTH);
    const_assert_eq!(25, ComponentValidationError::SELECT_MAXIMUM_VALUES_LIMIT);
    const_assert_eq!(
        1,
        ComponentValidationError::SELECT_MAXIMUM_VALUES_REQUIREMENT
    );
    const_assert_eq!(25, ComponentValidationError::SELECT_MINIMUM_VALUES_LIMIT);
    const_assert_eq!(25, ComponentValidationError::SELECT_OPTION_COUNT);
    const_assert_eq!(
        50,
        ComponentValidationError::SELECT_OPTION_DESCRIPTION_LENGTH
    );
    const_assert_eq!(25, ComponentValidationError::SELECT_OPTION_LABEL_LENGTH);
    const_assert_eq!(100, ComponentValidationError::SELECT_OPTION_VALUE_LENGTH);
    const_assert_eq!(100, ComponentValidationError::SELECT_PLACEHOLDER_LENGTH);
    const_assert_eq!(256, EmbedValidationError::AUTHOR_NAME_LENGTH);
    const_assert_eq!(2048, EmbedValidationError::DESCRIPTION_LENGTH);
    const_assert_eq!(6000, EmbedValidationError::EMBED_TOTAL_LENGTH);
    const_assert_eq!(25, EmbedValidationError::FIELD_COUNT);
    const_assert_eq!(256, EmbedValidationError::FIELD_NAME_LENGTH);
    const_assert_eq!(1024, EmbedValidationError::FIELD_VALUE_LENGTH);
    const_assert_eq!(2048, EmbedValidationError::FOOTER_TEXT_LENGTH);
    const_assert_eq!(256, EmbedValidationError::TITLE_LENGTH);

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
