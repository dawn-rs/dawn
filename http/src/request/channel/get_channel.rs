use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{channel::Channel, id::ChannelId};

/// Get a channel by its ID.
///
/// # Examples
///
/// Get channel `100`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::ChannelId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
///
/// let channel_id = ChannelId(100);
///
/// let channel = client.channel(channel_id).exec().await?;
/// # Ok(()) }
/// ```
pub struct GetChannel<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> GetChannel<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Channel> {
        let request = Request::from_route(Route::GetChannel {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
