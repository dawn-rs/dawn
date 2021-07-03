use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{
    application::command::permissions::GuildCommandPermissions,
    id::{ApplicationId, CommandId, GuildId},
};

/// Fetch command permissions for a command from the current application in a guild.
pub struct GetCommandPermissions<'a> {
    application_id: ApplicationId,
    command_id: CommandId,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetCommandPermissions<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Self {
        Self {
            application_id,
            command_id,
            guild_id,
            http,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<GuildCommandPermissions> {
        let request = Request::from_route(Route::GetCommandPermissions {
            application_id: self.application_id.0,
            command_id: self.command_id.0,
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
