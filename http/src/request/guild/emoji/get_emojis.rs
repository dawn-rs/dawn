use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{guild::Emoji, id::GuildId};

/// Get the emojis for a guild, by the guild's id.
///
/// # Examples
///
/// Get the emojis for guild `100`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = GuildId(100);
///
/// client.emojis(guild_id).exec().await?;
/// # Ok(()) }
/// ```
pub struct GetEmojis<'a> {
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetEmojis<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self { guild_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Emoji>> {
        let request = Request::from_route(Route::GetEmojis {
            guild_id: self.guild_id.0,
        });

        self.http.request(request)
    }
}
