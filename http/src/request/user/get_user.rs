use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::user::User;

/// Get a user's information by id.
pub struct GetUser<'a> {
    http: &'a Client,
    target_user: String,
}

impl<'a> GetUser<'a> {
    pub(crate) fn new(http: &'a Client, target_user: impl Into<String>) -> Self {
        Self {
            http,
            target_user: target_user.into(),
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<User> {
        let request = Request::from_route(Route::GetUser {
            target_user: self.target_user,
        });

        self.http.request(request)
    }
}
