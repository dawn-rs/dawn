use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, Pending, Request},
    routing::Route,
};
use hyper::body::Bytes;
use serde::de::DeserializeSeed;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_model::{
    guild::member::{Member, MemberDeserializer},
    id::GuildId,
};

#[cfg(not(feature = "simd-json"))]
use serde_json::Value;
#[cfg(feature = "simd-json")]
use simd_json::value::OwnedValue as Value;

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

struct SearchGuildMembersFields {
    query: String,
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
/// let client = Client::new("my token");
///
/// let guild_id = GuildId(100);
/// let members = client.search_guild_members(guild_id, String::from("Wumpus")).limit(10)?.await?;
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
    fields: SearchGuildMembersFields,
    fut: Option<Pending<'a, Bytes>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> SearchGuildMembers<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, query: impl Into<String>) -> Self {
        Self::_new(http, guild_id, query.into())
    }

    fn _new(http: &'a Client, guild_id: GuildId, query: String) -> Self {
        Self {
            fields: SearchGuildMembersFields { query, limit: None },
            fut: None,
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
        if !validate_inner::search_guild_members_limit(limit) {
            return Err(SearchGuildMembersError {
                kind: SearchGuildMembersErrorType::LimitInvalid { limit },
            });
        }

        self.fields.limit.replace(limit);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::from_route(Route::SearchGuildMembers {
            guild_id: self.guild_id.0,
            limit: self.fields.limit,
            query: self.fields.query.clone(),
        });

        self.fut.replace(Box::pin(self.http.request_bytes(request)));

        Ok(())
    }
}

impl Future for SearchGuildMembers<'_> {
    type Output = Result<Vec<Member>, HttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            self.as_mut().start()?;
        }

        let fut = self.fut.as_mut().expect("future is created");

        match fut.as_mut().poll(cx) {
            Poll::Ready(res) => {
                let bytes = res?;
                let mut members = Vec::new();

                let values =
                    crate::json::from_bytes::<Vec<Value>>(&bytes).map_err(HttpError::json)?;

                for value in values {
                    let member_deserializer = MemberDeserializer::new(self.guild_id);
                    members.push(
                        member_deserializer
                            .deserialize(value)
                            .map_err(HttpError::json)?,
                    );
                }

                Poll::Ready(Ok(members))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
