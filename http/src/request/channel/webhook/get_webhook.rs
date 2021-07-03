use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{channel::Webhook, id::WebhookId};

#[derive(Default)]
struct GetWebhookFields {
    token: Option<String>,
}

/// Get a webhook by ID.
pub struct GetWebhook<'a> {
    fields: GetWebhookFields,
    http: &'a Client,
    id: WebhookId,
}

impl<'a> GetWebhook<'a> {
    pub(crate) fn new(http: &'a Client, id: WebhookId) -> Self {
        Self {
            fields: GetWebhookFields::default(),
            http,
            id,
        }
    }

    /// Specify the token for auth, if not already authenticated with a Bot
    /// token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.fields.token.replace(token.into());

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Webhook> {
        let use_webhook_token = self.fields.token.is_some();

        let mut request = Request::builder(Route::GetWebhook {
            token: self.fields.token,
            webhook_id: self.id.0,
        });

        // If a webhook token has been configured, then we don't need to use
        // the client's authorization token.
        if use_webhook_token {
            request = request.use_authorization_token(false);
        }

        self.http.request(request.build())
    }
}
