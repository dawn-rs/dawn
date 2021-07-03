use super::Form;
use crate::{error::Error, routing::Route};
use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;

/// Builder to create a customized request.
///
/// # Examples
///
/// Create a request to create a message with a content of "test" in a
/// channel with an ID of 1:
///
/// ```
/// use twilight_http::{request::Request, routing::Route};
///
/// let body = br#"{
///     "content": "test"
/// }"#.to_vec();
///
/// let request = Request::builder(Route::CreateMessage {
///     channel_id: 1,
/// }).body(body).build();
/// ```
#[derive(Debug)]
pub struct RequestBuilder<'a>(Request<'a>);

impl<'a> RequestBuilder<'a> {
    /// Create a new request builder.
    #[must_use = "request has not been fully built"]
    pub const fn new(route: Route<'a>) -> Self {
        Self(Request::from_route(route))
    }

    /// Consume the builder, returning the built request.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use = "request information is not useful on its own and must be acted on"]
    pub fn build(self) -> Request<'a> {
        self.0
    }

    /// Set the contents of the body.
    #[must_use = "request has not been fully built"]
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.0.body.replace(body);

        self
    }

    /// Set the multipart form.
    #[must_use = "request has not been fully built"]
    pub fn form(mut self, form: Form) -> Self {
        self.0.form.replace(form);

        self
    }

    /// Set the headers to add.
    #[must_use = "request has not been fully built"]
    pub fn headers(mut self, iter: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self {
        self.0.headers.replace(iter.collect());

        self
    }

    /// Set the body, to be serialized as JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`ErrorType::Json`] error type if the value could not be
    /// serialized as JSON.
    ///
    /// [`ErrorType::Json`]: crate::error::ErrorType::Json
    #[must_use = "request has not been fully built"]
    pub fn json(self, to: &impl Serialize) -> Result<Self, Error> {
        let bytes = crate::json::to_vec(to).map_err(Error::json)?;

        Ok(self.body(bytes))
    }

    /// Whether to use the client's authorization token in the request, if one
    /// is set.
    ///
    /// This is primarily useful for executing webhooks.
    pub const fn use_authorization_token(mut self, use_authorization_token: bool) -> Self {
        self.0.use_authorization_token = use_authorization_token;

        self
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    /// The body of the request, if any.
    pub body: Option<Vec<u8>>,
    /// The multipart form of the request, if any.
    pub form: Option<Form>,
    /// The headers to set in the request, if any.
    pub headers: Option<HeaderMap<HeaderValue>>,
    /// The route of the request.
    pub route: Route<'a>,
    /// Whether to use the client's authorization token in the request.
    pub(crate) use_authorization_token: bool,
}

impl<'a> Request<'a> {
    /// Create a new request builder.
    ///
    /// # Examples
    ///
    /// Create a request to create a message with a content of "test" in a
    /// channel with an ID of 1:
    ///
    /// ```
    /// use twilight_http::{request::Request, routing::Route};
    ///
    /// let body = br#"{
    ///     "content": "test"
    /// }"#.to_vec();
    ///
    /// let request = Request::builder(Route::CreateMessage {
    ///     channel_id: 1,
    /// }).body(body).build();
    /// ```
    pub const fn builder(route: Route<'a>) -> RequestBuilder<'a> {
        RequestBuilder::new(route)
    }

    /// Create a request from only its route information.
    ///
    /// If you need to set additional configurations like the body then use
    /// [`builder`].
    ///
    /// # Examples
    ///
    /// Create a request to get a message with an ID of 2 in a channel with an
    /// ID of 1:
    ///
    /// ```
    /// use twilight_http::{request::Request, routing::Route};
    ///
    /// let request = Request::from_route(Route::GetMessage {
    ///     channel_id: 1,
    ///     message_id: 2,
    /// });
    /// ```
    ///
    /// [`builder`]: Self::builder
    pub const fn from_route(route: Route<'a>) -> Self {
        Self {
            body: None,
            form: None,
            headers: None,
            route,
            use_authorization_token: true,
        }
    }

    /// Whether to use the client's authorization token in the request.
    pub const fn use_authorization_token(&self) -> bool {
        self.use_authorization_token
    }
}

#[cfg(test)]
mod tests {
    use super::RequestBuilder;
    use static_assertions::assert_impl_all;
    use std::fmt::Debug;

    assert_impl_all!(RequestBuilder<'_>: Debug, Send, Sync);
}
