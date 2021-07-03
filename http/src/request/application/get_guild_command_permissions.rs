use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{
    application::command::permissions::GuildCommandPermissions,
    id::{ApplicationId, GuildId},
};

/// Get command permissions for all commands from the current application in a guild.
pub struct GetGuildCommandPermissions<'a> {
    application_id: ApplicationId,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildCommandPermissions<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
    ) -> Self {
        Self {
            application_id,
            guild_id,
            http,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<GuildCommandPermissions>> {
        let request = Request::from_route(Route::GetGuildCommandPermissions {
            application_id: self.application_id.0,
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
