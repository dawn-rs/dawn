use crate::{
    client::Client,
    request::Request,
    response::{marker::MemberBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{GuildId, UserId};

/// Get a member of a guild, by id.
pub struct GetMember<'a> {
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> GetMember<'a> {
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
    pub fn exec(self) -> ResponseFuture<MemberBody> {
        let request = Request::from_route(Route::GetMember {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        });

        let mut future = self.http.request(request);
        future.set_guild_id(self.guild_id);

        future
    }
}
