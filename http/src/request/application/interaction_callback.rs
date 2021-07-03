use crate::{
    client::Client,
    error::Error,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{application::callback::InteractionResponse, id::InteractionId};

/// Respond to an interaction, by ID and token.
pub struct InteractionCallback<'a> {
    interaction_id: InteractionId,
    interaction_token: &'a str,
    response: InteractionResponse,
    http: &'a Client,
}

impl<'a> InteractionCallback<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        interaction_id: InteractionId,
        interaction_token: &'a str,
        response: InteractionResponse,
    ) -> Self {
        Self {
            interaction_id,
            interaction_token,
            response,
            http,
        }
    }

    fn request(&self) -> Result<Request<'a>, Error> {
        Ok(Request::builder(Route::InteractionCallback {
            interaction_id: self.interaction_id.0,
            interaction_token: self.interaction_token,
        })
        .json(&self.response)?
        .build())
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        match self.request() {
            Ok(request) => self.http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}
