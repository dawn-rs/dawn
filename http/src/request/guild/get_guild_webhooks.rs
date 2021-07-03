use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{channel::Webhook, id::GuildId};

/// Get the webhooks of a guild.
pub struct GetGuildWebhooks<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildWebhooks<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Webhook>> {
        let request = Request::from_route(Route::GetGuildWebhooks {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
