use crate::{
    client::Client,
    request::{self, validate, AuditLogReason, AuditLogReasonError, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    id::{ApplicationId, ChannelId, UserId},
    invite::{Invite, TargetType},
};

/// Error created when an invite can not be created as configured.
#[derive(Debug)]
pub struct CreateInviteError {
    kind: CreateInviteErrorType,
}

impl CreateInviteError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateInviteErrorType {
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
    pub fn into_parts(self) -> (CreateInviteErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for CreateInviteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CreateInviteErrorType::MaxAgeTooOld { .. } => f.write_str("max age is too long"),
            CreateInviteErrorType::MaxUsesTooLarge { .. } => f.write_str("max uses is too many"),
        }
    }
}

impl Error for CreateInviteError {}

/// Type of [`CreateInviteError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateInviteErrorType {
    /// Configured maximum age is over 604800.
    MaxAgeTooOld,
    /// Configured maximum uses is over 100.
    MaxUsesTooLarge,
}

#[derive(Default, Serialize)]
struct CreateInviteFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_age: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_uses: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_application_id: Option<ApplicationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_type: Option<TargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unique: Option<bool>,
}

/// Create an invite, with options.
///
/// Requires the [`CREATE_INVITE`] permission.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
/// use twilight_model::id::ChannelId;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token".to_owned());
///
/// let channel_id = ChannelId(123);
/// let invite = client
///     .create_invite(channel_id)
///     .max_uses(3)?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`CREATE_INVITE`]: twilight_model::guild::Permissions::CREATE_INVITE
pub struct CreateInvite<'a> {
    channel_id: ChannelId,
    fields: CreateInviteFields,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> CreateInvite<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: CreateInviteFields::default(),
            http,
            reason: None,
        }
    }

    /// Set the maximum age for an invite.
    ///
    /// If no age is specified, Discord sets the age to 86400 seconds, or 24 hours.
    /// Set to 0 to never expire.
    ///
    /// # Examples
    ///
    /// Create an invite for a channel with a maximum of 1 use and an age of 1
    /// hour:
    ///
    /// ```rust,no_run
    /// use std::env;
    /// use twilight_http::Client;
    /// use twilight_model::id::ChannelId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// let invite = client.create_invite(ChannelId(1))
    ///     .max_age(60 * 60)?
    ///     .exec()
    ///     .await?
    ///     .model()
    ///     .await?;
    ///
    /// println!("invite code: {}", invite.code);
    /// # Ok(()) }
    /// ```
    pub fn max_age(mut self, max_age: u64) -> Result<Self, CreateInviteError> {
        if !validate::invite_max_age(max_age) {
            return Err(CreateInviteError {
                kind: CreateInviteErrorType::MaxAgeTooOld,
            });
        }

        self.fields.max_age.replace(max_age);

        Ok(self)
    }

    /// Set the maximum uses for an invite, or 0 for infinite.
    ///
    /// Discord defaults this to 0, or infinite.
    ///
    /// # Examples
    ///
    /// Create an invite for a channel with a maximum of 5 uses:
    ///
    /// ```rust,no_run
    /// use std::env;
    /// use twilight_http::Client;
    /// use twilight_model::id::ChannelId;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new(env::var("DISCORD_TOKEN")?);
    /// let invite = client.create_invite(ChannelId(1))
    ///     .max_uses(5)?
    ///     .exec()
    ///     .await?
    ///     .model()
    ///     .await?;
    ///
    /// println!("invite code: {}", invite.code);
    /// # Ok(()) }
    /// ```
    pub fn max_uses(mut self, max_uses: u64) -> Result<Self, CreateInviteError> {
        if !validate::invite_max_uses(max_uses) {
            return Err(CreateInviteError {
                kind: CreateInviteErrorType::MaxUsesTooLarge,
            });
        }

        self.fields.max_uses.replace(max_uses);

        Ok(self)
    }

    /// Set the target application ID for this invite.
    ///
    /// This only works if [`target_type`] is set to [`TargetType::EmbeddedApplication`].
    ///
    /// [`target_type`]: Self::target_type
    pub fn target_application_id(mut self, target_application_id: ApplicationId) -> Self {
        self.fields
            .target_application_id
            .replace(target_application_id);

        self
    }

    /// Set the target user id for this invite.
    pub fn target_user_id(mut self, target_user_id: UserId) -> Self {
        self.fields.target_user_id.replace(target_user_id);

        self
    }

    /// Set the target type for this invite.
    pub fn target_type(mut self, target_type: TargetType) -> Self {
        self.fields.target_type.replace(target_type);

        self
    }

    /// Specify true if the invite should grant temporary membership.
    ///
    /// Defaults to false.
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.fields.temporary.replace(temporary);

        self
    }

    /// Specify true if the invite should be unique. Defaults to false.
    ///
    /// If true, don't try to reuse a similar invite (useful for creating many unique one time use
    /// invites). Refer to [the discord docs] for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/resources/channel#create-channel-invite
    pub fn unique(mut self, unique: bool) -> Self {
        self.fields.unique.replace(unique);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Invite> {
        let mut request = Request::builder(Route::CreateInvite {
            channel_id: self.channel_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        if let Some(reason) = &self.reason {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason<'a> for CreateInvite<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::CreateInvite;
    use crate::Client;
    use std::error::Error;
    use twilight_model::id::ChannelId;

    #[test]
    fn test_max_age() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo".to_owned());
        let mut builder = CreateInvite::new(&client, ChannelId(1)).max_age(0)?;
        assert_eq!(Some(0), builder.fields.max_age);
        builder = builder.max_age(604_800)?;
        assert_eq!(Some(604_800), builder.fields.max_age);
        assert!(builder.max_age(604_801).is_err());

        Ok(())
    }

    #[test]
    fn test_max_uses() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo".to_owned());
        let mut builder = CreateInvite::new(&client, ChannelId(1)).max_uses(0)?;
        assert_eq!(Some(0), builder.fields.max_uses);
        builder = builder.max_uses(100)?;
        assert_eq!(Some(100), builder.fields.max_uses);
        assert!(builder.max_uses(101).is_err());

        Ok(())
    }
}
