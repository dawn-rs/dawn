pub mod application;
pub mod channel;
pub mod guild;
pub mod prelude;
pub mod template;
pub mod user;

mod audit_reason;
mod base;
mod get_gateway;
mod get_gateway_authed;
mod get_user_application;
mod get_voice_regions;
mod multipart;
mod validate;

pub use self::{
    audit_reason::{AuditLogReason, AuditLogReasonError},
    base::{Request, RequestBuilder},
    get_gateway::GetGateway,
    get_gateway_authed::GetGatewayAuthed,
    get_user_application::GetUserApplicationInfo,
    get_voice_regions::GetVoiceRegions,
    multipart::Form,
};

use crate::error::{Error, ErrorType};
use hyper::{
    header::{HeaderName, HeaderValue},
    Method as HyperMethod,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Serialize, Serializer};
use std::iter;

/// Request method.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Method {
    /// DELETE method.
    Delete,
    /// GET method.
    Get,
    /// PATCH method.
    Patch,
    /// POST method.
    Post,
    /// PUT method.
    Put,
}

impl Method {
    pub(crate) const fn into_hyper(self) -> HyperMethod {
        match self {
            Self::Delete => HyperMethod::DELETE,
            Self::Get => HyperMethod::GET,
            Self::Patch => HyperMethod::PATCH,
            Self::Post => HyperMethod::POST,
            Self::Put => HyperMethod::PUT,
        }
    }
}

/// Field that either serializes to null or a value.
///
/// This is particularly useful when combined with an `Option` by allowing three
/// states via `Option<NullableField<T>>`: undefined, null, and T.
///
/// When undefined a field can skip serialization, while if it's null then it will
/// serialize as null. This mechanism is primarily used in patch requests.
enum NullableField<T> {
    /// Remove a value.
    Null,
    /// Set a value.
    Value(T),
}

impl<T> NullableField<T> {
    /// Create a `NullableField` from an option.
    #[allow(clippy::missing_const_for_fn)]
    fn from_option(option: Option<T>) -> Self {
        match option {
            Some(value) => Self::Value(value),
            None => Self::Null,
        }
    }
}

impl<T: Serialize> Serialize for NullableField<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Value(inner) => serializer.serialize_some(inner),
        }
    }
}

pub(crate) fn audit_header(
    reason: &str,
) -> Result<impl Iterator<Item = (HeaderName, HeaderValue)>, Error> {
    let header_name = HeaderName::from_static("x-audit-log-reason");
    let encoded_reason = utf8_percent_encode(reason, NON_ALPHANUMERIC).to_string();
    let header_value = HeaderValue::from_str(&encoded_reason).map_err(|e| Error {
        kind: ErrorType::CreatingHeader {
            name: encoded_reason.clone(),
        },
        source: Some(Box::new(e)),
    })?;

    Ok(iter::once((header_name, header_value)))
}

#[cfg(test)]
mod tests {
    use super::Method;
    use hyper::Method as HyperMethod;
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(Method: Clone, Copy, Debug, Eq, PartialEq);

    #[test]
    fn test_method_conversions() {
        assert_eq!(HyperMethod::DELETE, Method::Delete.into_hyper());
        assert_eq!(HyperMethod::GET, Method::Get.into_hyper());
        assert_eq!(HyperMethod::PATCH, Method::Patch.into_hyper());
        assert_eq!(HyperMethod::POST, Method::Post.into_hyper());
        assert_eq!(HyperMethod::PUT, Method::Put.into_hyper());
    }
}
