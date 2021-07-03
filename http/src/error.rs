use crate::{api_error::ApiError, json::JsonError, response::StatusCode};
use hyper::{Body, Response};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub struct Error {
    pub(super) source: Option<Box<dyn StdError + Send + Sync>>,
    pub(super) kind: ErrorType,
}

impl Error {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (ErrorType, Option<Box<dyn StdError + Send + Sync>>) {
        (self.kind, self.source)
    }

    pub(super) fn json(source: JsonError) -> Self {
        Self {
            kind: ErrorType::Json,
            source: Some(Box::new(source)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ErrorType::BuildingRequest => f.write_str("failed to build the request"),
            ErrorType::ChunkingResponse => f.write_str("Chunking the response failed"),
            ErrorType::CreatingHeader { name, .. } => {
                f.write_str("Parsing the value for header {}")?;
                f.write_str(name)?;

                f.write_str(" failed")
            }
            ErrorType::Json => f.write_str("Given value couldn't be serialized"),
            ErrorType::Parsing { body, .. } => {
                f.write_str("Response body couldn't be deserialized: ")?;

                Debug::fmt(body, f)
            }
            ErrorType::RequestCanceled => {
                f.write_str("Request was canceled either before or while being sent")
            }
            ErrorType::RequestError => f.write_str("Parsing or sending the response failed"),
            ErrorType::RequestTimedOut => f.write_str("request timed out"),
            ErrorType::Response { error, status, .. } => {
                f.write_str("Response error: status code ")?;
                Display::fmt(status, f)?;
                f.write_str(", error: ")?;

                Display::fmt(error, f)
            }
            ErrorType::ServiceUnavailable { .. } => {
                f.write_str("api may be temporarily unavailable (received a 503)")
            }
            ErrorType::Unauthorized => {
                f.write_str("token in use is invalid, expired, or is revoked")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
#[non_exhaustive]
/// Type of [`Error`] that occurred.
pub enum ErrorType {
    BuildingRequest,
    ChunkingResponse,
    CreatingHeader {
        name: String,
    },
    Json,
    Parsing {
        body: Vec<u8>,
    },
    RequestCanceled,
    RequestError,
    RequestTimedOut,
    Response {
        body: Vec<u8>,
        error: ApiError,
        status: StatusCode,
    },
    /// API service is unavailable. Consider re-sending the request at a
    /// later time.
    ///
    /// This may occur during Discord API stability incidents.
    ServiceUnavailable {
        response: Response<Body>,
    },
    /// Token in use has become revoked or is otherwise invalid.
    ///
    /// This can occur if a bot token is invalidated or an access token expires
    /// or is revoked. Recreate the client to configure a new token.
    Unauthorized,
}
