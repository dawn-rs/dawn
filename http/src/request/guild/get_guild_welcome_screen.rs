use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{id::GuildId, invite::WelcomeScreen};

/// Get the guild's welcome screen.
pub struct GetGuildWelcomeScreen<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildWelcomeScreen<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<WelcomeScreen> {
        let request = Request::from_route(Route::GetGuildWelcomeScreen {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
