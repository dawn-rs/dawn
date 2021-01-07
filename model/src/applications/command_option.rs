use std::convert::From;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// An option for a [`Command`](crate::applications::Command).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CommandOption {
    SubCommand(OptionsCommandOptionData),
    SubCommandGroup(OptionsCommandOptionData),
    String(ChoiceCommandOptionData),
    Integer(ChoiceCommandOptionData),
    Boolean(BaseCommandOptionData),
    User(BaseCommandOptionData),
    Channel(BaseCommandOptionData),
    Role(BaseCommandOptionData),
}

impl CommandOption {
    pub fn kind(&self) -> CommandOptionType {
        match self {
            CommandOption::SubCommand(_) => CommandOptionType::SubCommand,
            CommandOption::SubCommandGroup(_) => CommandOptionType::SubCommandGroup,
            CommandOption::String(_) => CommandOptionType::String,
            CommandOption::Integer(_) => CommandOptionType::Integer,
            CommandOption::Boolean(_) => CommandOptionType::Boolean,
            CommandOption::User(_) => CommandOptionType::User,
            CommandOption::Channel(_) => CommandOptionType::Channel,
            CommandOption::Role(_) => CommandOptionType::Role,
        }
    }
}

impl<'de> Deserialize<'de> for CommandOption {
    fn deserialize<D>(deserializer: D) -> Result<CommandOption, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(CommandOptionEnvelope::deserialize(deserializer)?.into())
    }
}

impl Serialize for CommandOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let base = match self {
            CommandOption::SubCommand(d) | CommandOption::SubCommandGroup(d) => d.base(),
            CommandOption::String(d) | CommandOption::Integer(d) => d.base(),
            CommandOption::Boolean(d)
            | CommandOption::User(d)
            | CommandOption::Channel(d)
            | CommandOption::Role(d) => d.clone(),
        };
        let choices = match self {
            CommandOption::String(d) | CommandOption::Integer(d) => Some(d.choices.clone()),
            _ => None,
        };
        let options = match self {
            CommandOption::SubCommand(d) | CommandOption::SubCommandGroup(d) => {
                Some(d.options.clone())
            }
            _ => None,
        };

        CommandOptionEnvelope {
            kind: self.kind(),
            name: base.name,
            description: base.description,
            default: base.default,
            required: base.required,
            choices,
            options,
        }
        .serialize(serializer)
    }
}

impl From<CommandOptionEnvelope> for CommandOption {
    fn from(mut envelope: CommandOptionEnvelope) -> Self {
        match envelope.kind {
            CommandOptionType::SubCommand => {
                CommandOption::SubCommand(OptionsCommandOptionData::from((
                    envelope.options.take().unwrap_or_default(),
                    envelope.into(),
                )))
            }
            CommandOptionType::SubCommandGroup => {
                CommandOption::SubCommandGroup(OptionsCommandOptionData::from((
                    envelope.options.take().unwrap_or_default(),
                    envelope.into(),
                )))
            }
            CommandOptionType::String => CommandOption::String(ChoiceCommandOptionData::from((
                envelope.choices.take().unwrap_or_default(),
                envelope.into(),
            ))),
            CommandOptionType::Integer => CommandOption::Integer(ChoiceCommandOptionData::from((
                envelope.choices.take().unwrap_or_default(),
                envelope.into(),
            ))),
            CommandOptionType::Boolean => CommandOption::Boolean(envelope.into()),
            CommandOptionType::User => CommandOption::User(envelope.into()),
            CommandOptionType::Channel => CommandOption::Channel(envelope.into()),
            CommandOptionType::Role => CommandOption::Role(envelope.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BaseCommandOptionData {
    /// The name of the option. It must be 32 characters or less.
    pub name: String,
    /// A description of the option. It must be 100 characters or less.
    pub description: String,
    /// The first required option that you wish the user to complete. Only one
    /// CommandOption may be default per command.
    ///
    /// For example, given a simple kick command:
    ///     `/kick @user [reason]`
    /// You would make the `@user` option default, as it's the first thing you
    /// would like the user to complete.
    ///
    /// NOTE: THIS IS CURRENTLY BROKEN. IT ALWAYS ERRORS WHEN SET.
    #[serde(default)]
    pub default: bool,
    /// Whether or not the option is required to be completed by a user.
    #[serde(default)]
    pub required: bool,
}

impl From<CommandOptionEnvelope> for BaseCommandOptionData {
    fn from(envelope: CommandOptionEnvelope) -> Self {
        Self {
            name: envelope.name,
            description: envelope.description,
            default: envelope.default,
            required: envelope.required,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct OptionsCommandOptionData {
    /// The name of the option. It must be 32 characters or less.
    pub name: String,
    /// A description of the option. It must be 100 characters or less.
    pub description: String,
    /// The first required option that you wish the user to complete. Only one
    /// CommandOption may be default per command.
    ///
    /// For example, given a simple kick command:
    ///     `/kick @user [reason]`
    /// You would make the `@user` option default, as it's the first thing you
    /// would like the user to complete.
    ///
    /// NOTE: THIS IS CURRENTLY BROKEN. IT ALWAYS ERRORS WHEN SET.
    #[serde(default)]
    pub default: bool,
    /// Whether or not the option is required to be completed by a user.
    #[serde(default)]
    pub required: bool,

    /// Used for specifying the nested options in a SubCommand or
    /// SubCommandGroup.
    #[serde(default)]
    pub options: Vec<CommandOption>,
}

impl From<(Vec<CommandOption>, BaseCommandOptionData)> for OptionsCommandOptionData {
    fn from(opt: (Vec<CommandOption>, BaseCommandOptionData)) -> Self {
        Self {
            name: opt.1.name,
            description: opt.1.description,
            default: opt.1.default,
            required: opt.1.required,
            options: opt.0,
        }
    }
}

impl OptionsCommandOptionData {
    fn base(&self) -> BaseCommandOptionData {
        BaseCommandOptionData {
            name: self.name.clone(),
            description: self.description.clone(),
            default: self.default,
            required: self.required,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChoiceCommandOptionData {
    /// The name of the option. It must be 32 characters or less.
    pub name: String,
    /// A description of the option. It must be 100 characters or less.
    pub description: String,
    /// The first required option that you wish the user to complete. Only one
    /// CommandOption may be default per command.
    ///
    /// For example, given a simple kick command:
    ///     `/kick @user [reason]`
    /// You would make the `@user` option default, as it's the first thing you
    /// would like the user to complete.
    ///
    /// NOTE: THIS IS CURRENTLY BROKEN. IT ALWAYS ERRORS WHEN SET.
    #[serde(default)]
    pub default: bool,
    /// Whether or not the option is required to be completed by a user.
    #[serde(default)]
    pub required: bool,

    /// Predetermined choices may be defined for a user to select. When
    /// completing this option, the user is prompted with a selector of all
    /// available choices.
    #[serde(default)]
    pub choices: Vec<CommandOptionChoice>,
}

impl From<(Vec<CommandOptionChoice>, BaseCommandOptionData)> for ChoiceCommandOptionData {
    fn from(opt: (Vec<CommandOptionChoice>, BaseCommandOptionData)) -> Self {
        Self {
            name: opt.1.name,
            description: opt.1.description,
            default: opt.1.default,
            required: opt.1.required,
            choices: opt.0,
        }
    }
}

impl ChoiceCommandOptionData {
    fn base(&self) -> BaseCommandOptionData {
        BaseCommandOptionData {
            name: self.name.clone(),
            description: self.description.clone(),
            default: self.default,
            required: self.required,
        }
    }
}

/// Specifies an option that a user must choose from in a dropdown.
///
/// Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#applicationcommandoptionchoice
#[serde(untagged)]
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum CommandOptionChoice {
    String { name: String, value: String },
    Int { name: String, value: i64 },
}

/// CommandOption is an option that can be supplied to an ApplicationCommand, or
/// nested under another CommandOption of type SubCommand or SubCommandGroup.
///
/// Choices and options are mutually exclusive.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct CommandOptionEnvelope {
    #[serde(rename = "type")]
    pub kind: CommandOptionType,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Option<Vec<CommandOptionChoice>>,
    #[serde(default)]
    pub options: Option<Vec<CommandOption>>,
}

/// CommandOptionType specifies the type of a CommandOption.
#[derive(
    Clone, Copy, Debug, Deserialize_repr, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize_repr,
)]
#[repr(u8)]
pub enum CommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
}
