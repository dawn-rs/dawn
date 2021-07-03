use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{channel::GuildChannel, id::GuildId};

/// Get the channels in a guild.
pub struct GetGuildChannels<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildChannels<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<GuildChannel>> {
        let request = Request::from_route(Route::GetChannels {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
