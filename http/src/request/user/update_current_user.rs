use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, NullableField, Pending, Request},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::user::User;

/// The error created when the user can not be updated as configured.
#[derive(Debug)]
pub struct UpdateCurrentUserError {
    kind: UpdateCurrentUserErrorType,
}

impl UpdateCurrentUserError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateCurrentUserErrorType {
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
        UpdateCurrentUserErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for UpdateCurrentUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateCurrentUserErrorType::UsernameInvalid { .. } => {
                f.write_str("the username length is invalid")
            }
        }
    }
}

impl Error for UpdateCurrentUserError {}

/// Type of [`UpdateCurrentUserError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateCurrentUserErrorType {
    /// The length of the username is either fewer than 2 UTF-16 characters or more than 32 UTF-16
    /// characters.
    UsernameInvalid {
        /// Provided username.
        username: String,
    },
}

#[derive(Default, Serialize)]
struct UpdateCurrentUserFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

/// Update the current user.
///
/// All paramaters are optional. If the username is changed, it may cause the discriminator to be
/// rnadomized.
pub struct UpdateCurrentUser<'a> {
    fields: UpdateCurrentUserFields,
    fut: Option<Pending<'a, User>>,
    http: &'a Client,
}

impl<'a> UpdateCurrentUser<'a> {
    pub(crate) fn new(http: &'a Client) -> Self {
        Self {
            fields: UpdateCurrentUserFields::default(),
            fut: None,
            http,
        }
    }

    /// Set the user's avatar.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn avatar(mut self, avatar: impl Into<Option<String>>) -> Self {
        self.fields
            .avatar
            .replace(NullableField::from_option(avatar.into()));

        self
    }

    /// Set the username.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 32 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateCurrentUserErrorType::UsernameInvalid`] error type if
    /// the username length is too short or too long.
    pub fn username(self, username: impl Into<String>) -> Result<Self, UpdateCurrentUserError> {
        self._username(username.into())
    }

    fn _username(mut self, username: String) -> Result<Self, UpdateCurrentUserError> {
        if !validate_inner::username(&username) {
            return Err(UpdateCurrentUserError {
                kind: UpdateCurrentUserErrorType::UsernameInvalid { username },
            });
        }

        self.fields.username.replace(username);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::UpdateCurrentUser)
            .json(&self.fields)?
            .build();

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(UpdateCurrentUser<'_>, User);
