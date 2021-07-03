use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{id::GuildId, invite::Invite};

/// Get information about the invites of a guild.
///
/// Requires the [`MANAGE_GUILD`] permission.
///
/// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
pub struct GetGuildInvites<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildInvites<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Invite>> {
        let request = Request::from_route(Route::GetGuildInvites {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
