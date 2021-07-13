use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, Pending, Request},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::guild::Guild;

#[derive(Debug)]
pub struct CreateGuildFromTemplateError {
    kind: CreateGuildFromTemplateErrorType,
}

impl CreateGuildFromTemplateError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CreateGuildFromTemplateErrorType {
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
        CreateGuildFromTemplateErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for CreateGuildFromTemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            CreateGuildFromTemplateErrorType::NameInvalid { .. } => {
                f.write_str("the guild name is invalid")
            }
        }
    }
}

impl Error for CreateGuildFromTemplateError {}

/// Type of [`CreateGuildFromTemplateError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateGuildFromTemplateErrorType {
    /// Name of the guild is either fewer than 2 UTF-16 characters or more than 100 UTF-16
    /// characters.
    NameInvalid {
        /// Provided name.
        name: String,
    },
}

#[derive(Serialize)]
struct CreateGuildFromTemplateFields {
    name: String,
    icon: Option<String>,
}

/// Create a new guild based on a template.
///
/// This endpoint can only be used by bots in less than 10 guilds.
///
/// # Errors
///
/// Returns a [`CreateGuildFromTemplateErrorType::NameInvalid`] error type if
/// the name is invalid.
pub struct CreateGuildFromTemplate<'a> {
    fields: CreateGuildFromTemplateFields,
    fut: Option<Pending<'a, Guild>>,
    http: &'a Client,
    template_code: String,
}

impl<'a> CreateGuildFromTemplate<'a> {
    pub(crate) fn new(
        http: &'a Client,
        template_code: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, CreateGuildFromTemplateError> {
        Self::_new(http, template_code.into(), name.into())
    }

    fn _new(
        http: &'a Client,
        template_code: String,
        name: String,
    ) -> Result<Self, CreateGuildFromTemplateError> {
        if !validate_inner::guild_name(&name) {
            return Err(CreateGuildFromTemplateError {
                kind: CreateGuildFromTemplateErrorType::NameInvalid { name },
            });
        }

        Ok(Self {
            fields: CreateGuildFromTemplateFields { name, icon: None },
            fut: None,
            http,
            template_code,
        })
    }

    /// Set the icon.
    ///
    /// This must be a Data URI, in the form of `data:image/{type};base64,{data}` where `{type}` is
    /// the image MIME type and `{data}` is the base64-encoded image. Refer to [the discord docs]
    /// for more information.
    ///
    /// [the discord docs]: https://discord.com/developers/docs/reference#image-data
    pub fn icon(self, icon: impl Into<String>) -> Self {
        self._icon(icon.into())
    }

    fn _icon(mut self, icon: String) -> Self {
        self.fields.icon.replace(icon);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::CreateGuildFromTemplate {
            template_code: self.template_code.clone(),
        })
        .json(&self.fields)?
        .build();

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(CreateGuildFromTemplate<'_>, Guild);
