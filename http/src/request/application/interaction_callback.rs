use crate::request::prelude::*;
use twilight_model::{application::callback::InteractionResponse, id::InteractionId};

/// Respond to an interaction, by ID and token.
pub struct InteractionCallback<'a> {
    interaction_id: InteractionId,
    interaction_token: String,
    response: InteractionResponse,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
}

impl<'a> InteractionCallback<'a> {
    pub(crate) fn new(
        http: &'a Client,
        interaction_id: InteractionId,
        interaction_token: impl Into<String>,
        response: InteractionResponse,
    ) -> Self {
        Self {
            interaction_id,
            interaction_token: interaction_token.into(),
            response,
            fut: None,
            http,
        }
    }

    fn start(&mut self) -> Result<()> {
        let request = Request::builder(Route::InteractionCallback {
            interaction_id: self.interaction_id.0,
            interaction_token: self.interaction_token.clone(),
        })
        .json(&self.response)?;

        self.fut
            .replace(Box::pin(self.http.verify(request.build())));

        Ok(())
    }
}

poll_req!(InteractionCallback<'_>, ());
