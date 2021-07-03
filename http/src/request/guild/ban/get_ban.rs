use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{
    guild::Ban,
    id::{GuildId, UserId},
};

/// Get information about a ban of a guild.
///
/// Includes the user banned and the reason.
pub struct GetBan<'a> {
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> GetBan<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            guild_id,
            http,
            user_id,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Ban> {
        let request = Request::from_route(Route::GetBan {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        });

        self.http.request(request)
    }
}
