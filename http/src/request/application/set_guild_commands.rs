use crate::{
    client::Client,
    error::Error,
    request::{Pending, Request},
    routing::Route,
};
use twilight_model::{
    application::command::Command,
    id::{ApplicationId, GuildId},
};

/// Set a guild's commands.
///
/// This method is idempotent: it can be used on every start, without being
/// ratelimited if there aren't changes to the commands.
pub struct SetGuildCommands<'a> {
    commands: Vec<Command>,
    application_id: ApplicationId,
    guild_id: GuildId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
}

impl<'a> SetGuildCommands<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        commands: Vec<Command>,
    ) -> Self {
        Self {
            commands,
            application_id,
            guild_id,
            fut: None,
            http,
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        let request = Request::builder(Route::SetGuildCommands {
            application_id: self.application_id.0,
            guild_id: self.guild_id.0,
        })
        .json(&self.commands)?;

        self.fut
            .replace(Box::pin(self.http.verify(request.build())));

        Ok(())
    }
}

poll_req!(SetGuildCommands<'_>, ());
