use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{guild::Role, id::GuildId};

/// Get the roles of a guild.
pub struct GetGuildRoles<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildRoles<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Role>> {
        let request = Request::from_route(Route::GetGuildRoles {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
