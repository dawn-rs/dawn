use crate::request::prelude::*;
use twilight_model::user::CurrentUser;

/// Get information about the current user.
pub struct GetCurrentUser<'a> {
    fut: Option<Pending<'a, CurrentUser>>,
    http: &'a Client,
}

impl<'a> GetCurrentUser<'a> {
    pub(crate) fn new(http: &'a Client) -> Self {
        Self { fut: None, http }
    }

    fn start(&mut self) -> Result<()> {
        let request = Request::from_route(Route::GetUser {
            target_user: "@me".to_owned(),
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetCurrentUser<'_>, CurrentUser);
