use crate::{
    client::Client,
    error::Error,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{
    application::command::Command,
    id::{ApplicationId, GuildId},
};

/// Set a guild's commands.
///
/// This method is idempotent: it can be used on every start, without being
/// ratelimited if there aren't changes to the commands.
pub struct SetGuildCommands<'a> {
    commands: &'a [Command],
    application_id: ApplicationId,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> SetGuildCommands<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        commands: &'a [Command],
    ) -> Self {
        Self {
            commands,
            application_id,
            guild_id,
            http,
        }
    }

    fn request(&self) -> Result<Request<'a>, Error> {
        Ok(Request::builder(Route::SetGuildCommands {
            application_id: self.application_id.0,
            guild_id: self.guild_id.0,
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
