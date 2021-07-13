use crate::{
    client::Client,
    error::Error,
    request::{
        application::{InteractionError, InteractionErrorType},
        validate_inner, Pending, Request,
    },
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    application::command::permissions::CommandPermissions,
    id::{ApplicationId, CommandId, GuildId},
};

#[derive(Serialize)]
struct UpdateCommandPermissionsFields {
    pub permissions: Vec<CommandPermissions>,
}

/// Update command permissions for a single command in a guild.
///
/// # Note:
///
/// This overwrites the command permissions so the full set of permissions
/// have to be sent every time.
pub struct UpdateCommandPermissions<'a> {
    application_id: ApplicationId,
    command_id: CommandId,
    guild_id: GuildId,
    fields: UpdateCommandPermissionsFields,
    fut: Option<Pending<'a, Vec<CommandPermissions>>>,
    http: &'a Client,
}

impl<'a> UpdateCommandPermissions<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
        permissions: Vec<CommandPermissions>,
    ) -> Result<Self, InteractionError> {
        if !validate_inner::command_permissions(permissions.len()) {
            return Err(InteractionError {
                kind: InteractionErrorType::TooManyCommandPermissions,
            });
        }

        Ok(Self {
            application_id,
            command_id,
            guild_id,
            fields: UpdateCommandPermissionsFields { permissions },
            fut: None,
            http,
        })
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::builder(Route::UpdateCommandPermissions {
            application_id: self.application_id.0,
            command_id: self.command_id.0,
            guild_id: self.guild_id.0,
        })
        .json(&self.fields)?;

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

poll_req!(UpdateCommandPermissions<'_>, Vec<CommandPermissions>);
