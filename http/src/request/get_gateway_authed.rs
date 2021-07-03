use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::gateway::connection_info::BotConnectionInfo;

/// Get information about the gateway, authenticated as a bot user.
///
/// Returns additional information: the recommended number of shards to use, and information on
/// the current session start limit.
pub struct GetGatewayAuthed<'a> {
    http: &'a Client,
}

impl<'a> GetGatewayAuthed<'a> {
    pub(crate) const fn new(http: &'a Client) -> Self {
        Self { http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<BotConnectionInfo> {
        let request = Request::from_route(Route::GetGatewayBot);

        self.http.request(request)
    }
}
