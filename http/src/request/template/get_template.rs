use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::template::Template;

/// Get a template by its code.
pub struct GetTemplate<'a> {
    http: &'a Client,
    template_code: String,
}

impl<'a> GetTemplate<'a> {
    pub(crate) fn new(http: &'a Client, template_code: impl Into<String>) -> Self {
        Self::_new(http, template_code.into())
    }

    const fn _new(http: &'a Client, template_code: String) -> Self {
        Self {
            http,
            template_code,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Template> {
        let request = Request::from_route(Route::GetTemplate {
            template_code: self.template_code,
        });

        self.http.request(request)
    }
}
