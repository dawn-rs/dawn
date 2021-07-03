use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{id::GuildId, template::Template};

/// Sync a template to the current state of the guild, by ID and code.
pub struct SyncTemplate<'a> {
    guild_id: GuildId,
    http: &'a Client,
    template_code: String,
}

impl<'a> SyncTemplate<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: GuildId,
        template_code: impl Into<String>,
    ) -> Self {
        Self::_new(http, guild_id, template_code.into())
    }

    const fn _new(http: &'a Client, guild_id: GuildId, template_code: String) -> Self {
        Self {
            guild_id,
            http,
            template_code,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Template> {
        let request = Request::from_route(Route::SyncTemplate {
            guild_id: self.guild_id.0,
            template_code: self.template_code,
        });

        self.http.request(request)
    }
}
