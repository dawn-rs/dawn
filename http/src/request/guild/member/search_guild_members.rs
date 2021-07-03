use crate::{
    client::Client,
    request::{validate, Request},
    response::{marker::MemberListBody, ResponseFuture},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::GuildId;

/// The error created when the members can not be queried as configured.
#[derive(Debug)]
pub struct SearchGuildMembersError {
    kind: SearchGuildMembersErrorType,
}

impl SearchGuildMembersError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &SearchGuildMembersErrorType {
        &self.kind
    }

    /// Consumes the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type nad the source error.
    #[must_use = "consuming the error int its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        SearchGuildMembersErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for SearchGuildMembersError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            SearchGuildMembersErrorType::LimitInvalid { .. } => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for SearchGuildMembersError {}

/// Type of [`SearchGuildMembersError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum SearchGuildMembersErrorType {
    /// The limit is either 0 or more than 1000.
    LimitInvalid {
        /// Provided limit.
        limit: u64,
    },
}

struct SearchGuildMembersFields<'a> {
    query: &'a str,
    limit: Option<u64>,
}

/// Search the members of a specific guild by a query.
///
/// The upper limit to this request is 1000. Discord defaults the limit to 1.
///
/// # Examples
///
/// Get the first 10 members of guild `100` matching `Wumpus`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::GuildId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = GuildId(100);
/// let members = client.search_guild_members(guild_id, "Wumpus").limit(10)?.exec().await?;
/// # Ok(()) }
/// ```
///
/// # Errors
///
/// Returns a [`SearchGuildMembersErrorType::LimitInvalid`] error type if the
/// limit is invalid.
///
/// [`GUILD_MEMBERS`]: twilight_model::gateway::Intents#GUILD_MEMBERS
pub struct SearchGuildMembers<'a> {
    fields: SearchGuildMembersFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> SearchGuildMembers<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, query: &'a str) -> Self {
        Self {
            fields: SearchGuildMembersFields { query, limit: None },
            guild_id,
            http,
        }
    }

    /// Sets the number of members to retrieve per request.
    ///
    /// The limit must be greater than 0 and less than 1000.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchGuildMembersErrorType::LimitInvalid`] error type if
    /// the limit is 0 or greater than 1000.
    pub fn limit(mut self, limit: u64) -> Result<Self, SearchGuildMembersError> {
        // Using get_guild_members_limit here as the limits are the same
        // and this endpoint is not officially documented yet.
        if !validate::search_guild_members_limit(limit) {
            return Err(SearchGuildMembersError {
                kind: SearchGuildMembersErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<MemberListBody> {
        let request = Request::from_route(Route::SearchGuildMembers {
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            query: self.fields.query,
        });

        let mut future = self.http.request(request);
        future.set_guild_id(self.guild_id);

        future
    }
}
