use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{guild::GuildWidget, id::GuildId};

/// Get the guild widget.
///
/// Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#get-guild-widget
pub struct GetGuildWidget<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildWidget<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<GuildWidget> {
        let request = Request::from_route(Route::GetGuildWidget {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
