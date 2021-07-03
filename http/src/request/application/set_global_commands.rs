use crate::{
    client::Client,
    error::Error,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{application::command::Command, id::ApplicationId};

/// Set global commands.
///
/// This method is idempotent: it can be used on every start, without being
/// ratelimited if there aren't changes to the commands.
pub struct SetGlobalCommands<'a> {
    commands: Vec<Command>,
    application_id: ApplicationId,
    http: &'a Client,
}

impl<'a> SetGlobalCommands<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        commands: Vec<Command>,
    ) -> Self {
        Self {
            commands,
            application_id,
            http,
        }
    }

    fn request(&self) -> Result<Request, Error> {
        Ok(Request::builder(Route::SetGlobalCommands {
            application_id: self.application_id.0,
        })
        .json(&self.commands)?
        .build())
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        match self.request() {
            Ok(request) => self.http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}
