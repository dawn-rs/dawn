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
use twilight_model::id::{GuildId, UserId};

/// The error created when the members can not be fetched as configured.
#[derive(Debug)]
pub struct GetGuildMembersError {
    kind: GetGuildMembersErrorType,
}

impl GetGuildMembersError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetGuildMembersErrorType {
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
        GetGuildMembersErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetGuildMembersError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetGuildMembersErrorType::LimitInvalid { .. } => f.write_str("the limit is invalid"),
        }
    }
}

impl Error for GetGuildMembersError {}

/// Type of [`GetGuildMembersError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetGuildMembersErrorType {
    /// The limit is either 0 or more than 1000.
    LimitInvalid {
        /// Provided limit.
        limit: u64,
    },
}

struct GetGuildMembersFields {
    after: Option<UserId>,
    limit: Option<u64>,
    presences: Option<bool>,
}

/// Get the members of a guild, by id.
///
/// The upper limit to this request is 1000. If more than 1000 members are needed, the requests
/// must be chained. Discord defaults the limit to 1.
///
/// # Examples
///
/// Get the first 500 members of guild `100` after user ID `3000`:
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::{GuildId, UserId};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let guild_id = GuildId(100);
/// let user_id = UserId(3000);
/// let members = client.guild_members(guild_id).after(user_id).exec().await?;
/// # Ok(()) }
/// ```
pub struct GetGuildMembers<'a> {
    fields: GetGuildMembersFields,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildMembers<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: GetGuildMembersFields {
                after: None,
                limit: None,
                presences: None,
            },
            guild_id,
            http,
        }
    }

    /// Sets the user ID to get members after.
    pub const fn after(mut self, after: UserId) -> Self {
        self.fields.after = Some(after);

        self
    }

    /// Sets the number of members to retrieve per request.
    ///
    /// The limit must be greater than 0 and less than 1000.
    ///
    /// # Errors
    ///
    /// Returns a [`GetGuildMembersErrorType::LimitInvalid`] error type if the
    /// limit is 0 or greater than 1000.
    pub const fn limit(mut self, limit: u64) -> Result<Self, GetGuildMembersError> {
        if !validate::get_guild_members_limit(limit) {
            return Err(GetGuildMembersError {
                kind: GetGuildMembersErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit = Some(limit);

        Ok(self)
    }

    /// Sets whether to retrieve matched member presences
    pub const fn presences(mut self, presences: bool) -> Self {
        self.fields.presences = Some(presences);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<MemberListBody> {
        let request = Request::from_route(Route::GetGuildMembers {
            after: self.fields.after.map(|x| x.0),
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            presences: self.fields.presences,
        });

        let mut future = self.http.request(request);
        future.set_guild_id(self.guild_id);

        future
    }
}
