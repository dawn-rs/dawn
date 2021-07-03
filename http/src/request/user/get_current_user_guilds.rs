use crate::{
    client::Client,
    request::{validate, Request},
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{id::GuildId, user::CurrentUserGuild};

/// The error created when the current guilds can not be retrieved as configured.
#[derive(Debug)]
pub struct GetCurrentUserGuildsError {
    kind: GetCurrentUserGuildsErrorType,
}

impl GetCurrentUserGuildsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetCurrentUserGuildsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        GetCurrentUserGuildsErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetCurrentUserGuildsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetCurrentUserGuildsErrorType::LimitInvalid => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetCurrentUserGuildsError {}

/// Type of [`GetCurrentUserGuildsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetCurrentUserGuildsErrorType {
    /// The maximum number of guilds to retrieve is 0 or more than 100.
    LimitInvalid,
}

struct GetCurrentUserGuildsFields {
    after: Option<GuildId>,
    before: Option<GuildId>,
    limit: Option<u64>,
}

/// Returns a list of guilds for the current user.
///
/// # Examples
///
/// Get the first 25 guilds with an ID after `300` and before
/// `400`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let after = GuildId(300);
/// let before = GuildId(400);
/// let guilds = client.current_user_guilds()
///     .after(after)
///     .before(before)
///     .limit(25)?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct GetCurrentUserGuilds<'a> {
    fields: GetCurrentUserGuildsFields,
    http: &'a Client,
}

impl<'a> GetCurrentUserGuilds<'a> {
    pub(crate) const fn new(http: &'a Client) -> Self {
        Self {
            fields: GetCurrentUserGuildsFields {
                after: None,
                before: None,
                limit: None,
            },
            http,
        }
    }

    /// Get guilds after this guild id.
    pub const fn after(mut self, guild_id: GuildId) -> Self {
        self.fields.after = Some(guild_id);

        self
    }

    /// Get guilds before this guild id.
    pub const fn before(mut self, guild_id: GuildId) -> Self {
        self.fields.before = Some(guild_id);

        self
    }

    /// Set the maximum number of guilds to retrieve.
    ///
    /// The minimum is 1 and the maximum is 100. Refer to [the discord docs] for more information.
    ///
    /// # Errors
    ///
    /// Returns a [`GetCurrentUserGuildsErrorType::LimitInvalid`] error type if
    /// the amount is greater than 100.
    ///
    /// [the discord docs]: https://discordapp.com/developers/docs/resources/user#get-current-user-guilds-query-string-params
    pub const fn limit(mut self, limit: u64) -> Result<Self, GetCurrentUserGuildsError> {
        if !validate::get_current_user_guilds_limit(limit) {
            return Err(GetCurrentUserGuildsError {
                kind: GetCurrentUserGuildsErrorType::LimitInvalid,
            });
        }

        self.fields.limit = Some(limit);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<CurrentUserGuild>> {
        let request = Request::from_route(Route::GetGuilds {
            after: self.fields.after.map(|x| x.0),
            before: self.fields.before.map(|x| x.0),
            limit: self.fields.limit,
        });

        self.http.request(request)
    }
}
