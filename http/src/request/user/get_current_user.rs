use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::user::CurrentUser;

/// Get information about the current user.
pub struct GetCurrentUser<'a> {
    http: &'a Client,
}

impl<'a> GetCurrentUser<'a> {
    pub(crate) const fn new(http: &'a Client) -> Self {
        Self { http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<CurrentUser> {
        let request = Request::from_route(Route::GetCurrentUser);

        self.http.request(request)
    }
}
