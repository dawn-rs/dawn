use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::voice::VoiceRegion;

/// Get a list of voice regions that can be used when creating a guild.
pub struct GetVoiceRegions<'a> {
    http: &'a Client,
}

impl<'a> GetVoiceRegions<'a> {
    pub(crate) const fn new(http: &'a Client) -> Self {
        Self { http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<VoiceRegion>> {
        let request = Request::from_route(Route::GetVoiceRegions);

        self.http.request(request)
    }
}
