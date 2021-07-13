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
use twilight_model::{id::GuildId, template::Template};

/// Error emitted when the template can not be upated as configured.
#[derive(Debug)]
pub struct UpdateTemplateError {
    kind: UpdateTemplateErrorType,
}

impl UpdateTemplateError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateTemplateErrorType {
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
        UpdateTemplateErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for UpdateTemplateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            UpdateTemplateErrorType::NameInvalid { .. } => {
                f.write_str("the template name is invalid")
            }
            UpdateTemplateErrorType::DescriptionTooLarge { .. } => {
                f.write_str("the template description is too large")
            }
        }
    }
}

impl Error for UpdateTemplateError {}

/// Type of [`UpdateTemplateError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateTemplateErrorType {
    /// Name of the template is invalid.
    NameInvalid {
        /// Provided name.
        name: String,
    },
    /// Description of the template is invalid.
    DescriptionTooLarge {
        /// Provided description.
        description: String,
    },
}

#[derive(Serialize)]
struct UpdateTemplateFields {
    name: Option<String>,
    description: Option<String>,
}

/// Update the template's metadata, by ID and code.
pub struct UpdateTemplate<'a> {
    fields: UpdateTemplateFields,
    fut: Option<Pending<'a, Template>>,
    guild_id: GuildId,
    http: &'a Client,
    template_code: String,
}

impl<'a> UpdateTemplate<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: GuildId,
        template_code: impl Into<String>,
    ) -> Self {
        Self::_new(http, guild_id, template_code.into())
    }

    fn _new(http: &'a Client, guild_id: GuildId, template_code: String) -> Self {
        Self {
            fields: UpdateTemplateFields {
                name: None,
                description: None,
            },
            fut: None,
            guild_id,
            http,
            template_code,
        }
    }

    /// Set the description.
    ///
    /// This must be at most 120 characters in length.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateTemplateErrorType::DescriptionTooLarge`] error type
    /// if the description is too large.
    pub fn description(self, description: impl Into<String>) -> Result<Self, UpdateTemplateError> {
        self._description(description.into())
    }

    fn _description(mut self, description: String) -> Result<Self, UpdateTemplateError> {
        if !validate_inner::template_description(&description) {
            return Err(UpdateTemplateError {
                kind: UpdateTemplateErrorType::DescriptionTooLarge { description },
            });
        }

        self.fields.description.replace(description);

        Ok(self)
    }

    /// Set the name.
    ///
    /// This must be at least 1, and at most 100 characters in length.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateTemplateErrorType::NameInvalid`] error type if the
    /// name is invalid.
    pub fn name(self, name: impl Into<String>) -> Result<Self, UpdateTemplateError> {
        self._name(name.into())
    }

    fn _name(mut self, name: String) -> Result<Self, UpdateTemplateError> {
        if !validate_inner::template_name(&name) {
            return Err(UpdateTemplateError {
                kind: UpdateTemplateErrorType::NameInvalid { name },
            });
        }

        self.fields.name.replace(name);

        Ok(self)
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::UpdateTemplate {
            guild_id: self.guild_id.0,
            template_code: self.template_code.clone(),
        })
        .json(&self.fields)?
        .build();

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(UpdateTemplate<'_>, Template);
