use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{guild::VanityUrl, id::GuildId};

/// Get a guild's vanity url, if there is one.
pub struct GetGuildVanityUrl<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildVanityUrl<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<VanityUrl> {
        let request = Request::from_route(Route::GetGuildVanityUrl {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
