use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{guild::Ban, id::GuildId};

/// Retrieve the bans for a guild.
///
/// # Examples
///
/// Retrieve the bans for guild `1`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
///
/// let guild_id = GuildId(1);
///
/// let bans = client.bans(guild_id).exec().await?;
/// # Ok(()) }
/// ```
pub struct GetBans<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetBans<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Ban>> {
        let request = Request::from_route(Route::GetBans {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
