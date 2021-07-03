use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::ApplicationId;

/// Delete a original interaction response.
///
/// # Examples
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
/// use twilight_http::request::AuditLogReason;
/// use twilight_model::id::ApplicationId;
///
/// let client = Client::new(env::var("DISCORD_TOKEN")?);
/// client.set_application_id(ApplicationId(1));
///
/// client
///     .delete_interaction_original("token here")?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct DeleteOriginalResponse<'a> {
    application_id: ApplicationId,
    http: &'a Client,
    token: &'a str,
}

impl<'a> DeleteOriginalResponse<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: ApplicationId,
        token: &'a str,
    ) -> Self {
        Self {
            application_id,
            http,
            token,
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let request = Request::from_route(Route::DeleteInteractionOriginal {
            application_id: self.application_id.0,
            interaction_token: self.token,
        });

        self.http.request(request)
    }
}
