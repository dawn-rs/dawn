pub mod create_followup_message;

mod create_global_command;
mod create_guild_command;
mod delete_followup_message;
mod delete_global_command;
mod delete_guild_command;
mod delete_original_response;
mod get_command_permissions;
mod get_global_commands;
mod get_guild_command_permissions;
mod get_guild_commands;
mod get_original_response;
mod interaction_callback;
mod set_command_permissions;
mod set_global_commands;
mod set_guild_commands;
mod update_command_permissions;
mod update_followup_message;
mod update_global_command;
mod update_guild_command;
mod update_original_response;

pub use self::{
    create_followup_message::CreateFollowupMessage,
    create_global_command::CreateGlobalCommand,
    create_guild_command::CreateGuildCommand,
    delete_followup_message::DeleteFollowupMessage,
    delete_global_command::DeleteGlobalCommand,
    delete_guild_command::DeleteGuildCommand,
    delete_original_response::DeleteOriginalResponse,
    get_command_permissions::GetCommandPermissions,
    get_global_commands::GetGlobalCommands,
    get_guild_command_permissions::GetGuildCommandPermissions,
    get_guild_commands::GetGuildCommands,
    get_original_response::GetOriginalResponse,
    interaction_callback::InteractionCallback,
    set_command_permissions::SetCommandPermissions,
    set_global_commands::SetGlobalCommands,
    set_guild_commands::SetGuildCommands,
    update_command_permissions::UpdateCommandPermissions,
    update_followup_message::{
        UpdateFollowupMessage, UpdateFollowupMessageError, UpdateFollowupMessageErrorType,
    },
    update_global_command::UpdateGlobalCommand,
    update_guild_command::UpdateGuildCommand,
    update_original_response::{
        UpdateOriginalResponse, UpdateOriginalResponseError, UpdateOriginalResponseErrorType,
    },
};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::application::command::CommandOption;

/// The error created if the creation of interaction fails.
#[derive(Debug)]
pub struct InteractionError {
    pub(crate) kind: InteractionErrorType,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum InteractionErrorType {
    /// Application id was not set on the client.
    ApplicationIdNotPresent,
    /// Command name validation failed.
    CommandNameValidationFailed { name: String },
    /// Command description validation failed.
    CommandDescriptionValidationFailed { description: String },
    /// Required command options have to be passed before optional ones.
    CommandOptionsRequiredFirst { option: CommandOption },
    /// More than 10 permission overwrites were set.
    TooManyCommandPermissions,
}

impl InteractionError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &InteractionErrorType {
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
    pub fn into_parts(self) -> (InteractionErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for InteractionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            InteractionErrorType::ApplicationIdNotPresent => {
                f.write_str("application id not present")
            }
            InteractionErrorType::CommandNameValidationFailed { .. } => {
                f.write_str("command name must be between 3 and 32 characters")
            }
            InteractionErrorType::CommandDescriptionValidationFailed { .. } => {
                f.write_str("command description must be between 1 and 100 characters")
            }
            InteractionErrorType::CommandOptionsRequiredFirst { .. } => {
                f.write_str("optional command options must be added after required")
            }
            InteractionErrorType::TooManyCommandPermissions { .. } => {
                f.write_str("more than 10 permission overwrites were set")
            }
        }
    }
}

impl Error for InteractionError {}
