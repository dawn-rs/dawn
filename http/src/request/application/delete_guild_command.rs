use crate::request::prelude::*;
use twilight_model::id::{ApplicationId, CommandId, GuildId};

/// Delete a command in a guild, by ID.
pub struct DeleteGuildCommand<'a> {
    application_id: ApplicationId,
    command_id: CommandId,
    fut: Option<Pending<'a, ()>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> DeleteGuildCommand<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Self {
        Self {
            application_id,
            command_id,
            fut: None,
            guild_id,
            http,
        }
    }

    fn start(&mut self) -> Result<()> {
        let request = Request::from_route(Route::DeleteGuildCommand {
            application_id: self.application_id.0,
            command_id: self.command_id.0,
            guild_id: self.guild_id.0,
        });

        self.fut.replace(Box::pin(self.http.verify(request)));

        Ok(())
    }
}

poll_req!(DeleteGuildCommand<'_>, ());
