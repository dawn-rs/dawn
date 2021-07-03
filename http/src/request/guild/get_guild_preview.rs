use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{guild::GuildPreview, id::GuildId};

/// For public guilds, get the guild preview.
///
/// This works even if the user is not in the guild.
pub struct GetGuildPreview<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildPreview<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<GuildPreview> {
        let request = Request::from_route(Route::GetGuildPreview {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
