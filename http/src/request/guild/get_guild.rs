use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{guild::Guild, id::GuildId};

#[derive(Default)]
struct GetGuildFields {
    with_counts: bool,
}

/// Get information about a guild.
pub struct GetGuild<'a> {
    fields: GetGuildFields,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuild<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: GetGuildFields::default(),
            guild_id,
            http,
        }
    }

    /// Sets if you want to receive `approximate_member_count` and `approximate_presence_count` in
    /// the guild structure.
    pub const fn with_counts(mut self, with: bool) -> Self {
        self.fields.with_counts = with;

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Guild> {
        let request = Request::from_route(Route::GetGuild {
            guild_id: self.guild_id.0,
            with_counts: self.fields.with_counts,
        });

        self.http.request(request)
    }
}
