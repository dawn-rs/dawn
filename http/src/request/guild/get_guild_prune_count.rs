use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, Pending, Request},
    routing::Route,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::GuildPrune,
    id::{GuildId, RoleId},
};

/// The error created when the guild prune count can not be requested as configured.
#[derive(Debug)]
pub struct GetGuildPruneCountError {
    kind: GetGuildPruneCountErrorType,
}

impl GetGuildPruneCountError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &GetGuildPruneCountErrorType {
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
        GetGuildPruneCountErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for GetGuildPruneCountError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            GetGuildPruneCountErrorType::DaysInvalid => {
                f.write_str("the number of days is invalid")
            }
        }
    }
}

impl Error for GetGuildPruneCountError {}

/// Type of [`GetGuildPruneCountError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum GetGuildPruneCountErrorType {
    /// The number of days is 0.
    DaysInvalid,
}

#[derive(Default)]
struct GetGuildPruneCountFields {
    days: Option<u64>,
    include_roles: Vec<u64>,
}

/// Get the counts of guild members to be pruned.
pub struct GetGuildPruneCount<'a> {
    fields: GetGuildPruneCountFields,
    fut: Option<Pending<'a, GuildPrune>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildPruneCount<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: GetGuildPruneCountFields::default(),
            fut: None,
            guild_id,
            http,
        }
    }

    /// Set the number of days that a user must be inactive before being
    /// able to be pruned.
    ///
    /// The number of days must be greater than 0, and less than or equal to 30.
    ///
    /// # Errors
    ///
    /// Returns a [`GetGuildPruneCountErrorType::DaysInvalid`] error type if the
    /// number of days is 0.
    pub fn days(mut self, days: u64) -> Result<Self, GetGuildPruneCountError> {
        if !validate_inner::guild_prune_days(days) {
            return Err(GetGuildPruneCountError {
                kind: GetGuildPruneCountErrorType::DaysInvalid,
            });
        }

        self.fields.days.replace(days);

        Ok(self)
    }

    /// List of roles to include when calculating prune count
    pub fn include_roles(mut self, roles: impl Iterator<Item = RoleId>) -> Self {
        let roles = roles.map(|e| e.0).collect::<Vec<_>>();

        self.fields.include_roles = roles;

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::from_route(Route::GetGuildPruneCount {
            days: self.fields.days,
            guild_id: self.guild_id.0,
            include_roles: self.fields.include_roles.clone(),
        });

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(GetGuildPruneCount<'_>, GuildPrune);

#[cfg(test)]
mod test {
    use super::GetGuildPruneCount;
    use crate::Client;
    use twilight_model::id::GuildId;

    #[test]
    fn test_days() {
        fn days_valid(days: u64) -> bool {
            let client = Client::new("");
            let count = GetGuildPruneCount::new(&client, GuildId(0));
            let days_result = count.days(days);
            days_result.is_ok()
        }

        assert!(!days_valid(0));
        assert!(days_valid(1));
        assert!(!days_valid(u64::max_value()));
    }
}
