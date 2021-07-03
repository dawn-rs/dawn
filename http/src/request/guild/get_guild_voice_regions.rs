use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{id::GuildId, voice::VoiceRegion};

/// Get voice region data for the guild.
///
/// Can return VIP servers if the guild is VIP-enabled.
pub struct GetGuildVoiceRegions<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildVoiceRegions<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<VoiceRegion>> {
        let request = Request::from_route(Route::GetGuildVoiceRegions {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
