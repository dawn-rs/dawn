use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{id::UserId, user::User};

/// Get a user's information by id.
pub struct GetUser<'a> {
    http: &'a Client,
    user_id: UserId,
}

impl<'a> GetUser<'a> {
    pub(crate) const fn new(http: &'a Client, user_id: UserId) -> Self {
        Self { http, user_id }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<User> {
        let request = Request::from_route(Route::GetUser {
            user_id: self.user_id.0,
        });

        self.http.request(request)
    }
}
