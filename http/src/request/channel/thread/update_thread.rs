use super::{ThreadValidationError, ThreadValidationErrorType};
use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, AuditLogReason, AuditLogReasonError, Pending, Request},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::{thread::AutoArchiveDuration, Channel},
    id::ChannelId,
};

#[derive(Default, Serialize)]
struct UpdateThreadFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<AutoArchiveDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_per_user: Option<u64>,
}

/// Update a thread.
///
/// All fields are optional. The minimum length of the name is 1 UTF-16
/// characters and the maximum is 100 UTF-16 characters.
pub struct UpdateThread<'a> {
    channel_id: ChannelId,
    fields: UpdateThreadFields,
    fut: Option<Pending<'a, Channel>>,
    http: &'a Client,
    reason: Option<String>,
}

impl<'a> UpdateThread<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            fields: UpdateThreadFields::default(),
            fut: None,
            http,
            reason: None,
        }
    }

    pub fn archived(mut self, archived: bool) -> Self {
        self.fields.archived.replace(archived);

        self
    }

    pub fn auto_archive_duration(mut self, auto_archive_duration: AutoArchiveDuration) -> Self {
        self.fields
            .auto_archive_duration
            .replace(auto_archive_duration);

        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.fields.locked.replace(locked);

        self
    }

    pub fn name(self, name: impl Into<String>) -> Result<Self, ThreadValidationError> {
        self._name(name.into())
    }

    fn _name(mut self, name: String) -> Result<Self, ThreadValidationError> {
        if !validate::channel_name(&name) {
            return Err(ThreadValidationError {
                kind: ThreadValidationErrorType::NameInvalid { name },
            });
        }

        self.fields.name.replace(name);

        Ok(self)
    }

    /// Set the number of seconds that a user must wait before before they are
    /// able to send another message.
    ///
    /// The minimum is 0 and the maximum is 21600. Refer to [the discord docs]
    /// for more details.  This is also known as "Slow Mode".
    ///
    /// # Errors
    ///
    /// Returns an [`ThreadValidationErrorType::RateLimitPerUserInvalid`] error type
    /// if the amount is greater than 21600.
    ///
    /// [the discord docs]: https://discordapp.com/developers/docs/resources/channel#channel-object-channel-structure>
    pub fn rate_limit_per_user(
        mut self,
        rate_limit_per_user: u64,
    ) -> Result<Self, ThreadValidationError> {
        if rate_limit_per_user > 21600 {
            return Err(ThreadValidationError {
                kind: ThreadValidationErrorType::RateLimitPerUserInvalid {
                    rate_limit_per_user,
                },
            });
        }

        self.fields.rate_limit_per_user.replace(rate_limit_per_user);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let mut request = Request::builder(Route::UpdateChannel {
            channel_id: self.channel_id.0,
        });

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        self.fut
            .replace(Box::pin(self.http.request(request.build())));

        Ok(())
    }
}

impl<'a> AuditLogReason for UpdateThread<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

poll_req!(UpdateThread<'_>, Channel);
