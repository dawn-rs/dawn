use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::id::{ApplicationId, MessageId};

/// Delete a followup message created from a interaction.
///
/// # Examples
///
/// ```no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::env;
/// use twilight_http::Client;
/// use twilight_model::id::{MessageId, ApplicationId};
///
/// let client = Client::new(env::var("DISCORD_TOKEN")?);
/// client
///     .delete_followup_message("token here", MessageId(2))?
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
pub struct DeleteFollowupMessage<'a> {
    http: &'a Client,
    message_id: MessageId,
    token: &'a str,
    application_id: ApplicationId,
}

impl<'a> DeleteFollowupMessage<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        application_id: ApplicationId,
        token: &'a str,
        message_id: MessageId,
    ) -> Self {
        Self {
            http,
            message_id,
            token,
            application_id,
        }
    }

    const fn request(self) -> Request<'a> {
        Request::from_route(Route::DeleteWebhookMessage {
            message_id: self.message_id.0,
            token: self.token,
            webhook_id: self.application_id.0,
        })
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        self.http.request(self.request())
    }
}

#[cfg(test)]
mod tests {
    use super::DeleteFollowupMessage;
    use crate::{client::Client, request::Request, routing::Route};
    use twilight_model::id::{ApplicationId, MessageId};

    #[test]
    fn test_request() {
        let client = Client::new("token".to_owned());

        let builder = DeleteFollowupMessage::new(&client, ApplicationId(1), "token", MessageId(2));
        let actual = builder.request();

        let expected = Request::from_route(Route::DeleteWebhookMessage {
            message_id: 2,
            token: "token",
            webhook_id: 1,
        });

        assert_eq!(expected.route, actual.route);
    }
}
