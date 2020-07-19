use crate::request::prelude::*;
use twilight_model::id::{GuildId, IntegrationId};

/// Delete an integration for a guild, by the integration's id.
pub struct DeleteGuildIntegration<'a> {
    fut: Option<Pending<'a, ()>>,
    guild_id: GuildId,
    http: &'a Client,
    integration_id: IntegrationId,
    reason: Option<String>,
}

impl<'a> DeleteGuildIntegration<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, integration_id: IntegrationId) -> Self {
        Self {
            fut: None,
            guild_id,
            http,
            integration_id,
            reason: None,
        }
    }

    /// Attach an audit log reason to this request.
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason.replace(reason.into());

        self
    }

    fn start(&mut self) -> Result<()> {
        let request = if let Some(reason) = &self.reason {
            let headers = audit_header(&reason)?;
            Request::from((
                headers,
                Route::DeleteGuildIntegration {
                    guild_id: self.guild_id.0,
                    integration_id: self.integration_id.0,
                },
            ))
        } else {
            Request::from(Route::DeleteGuildIntegration {
                guild_id: self.guild_id.0,
                integration_id: self.integration_id.0,
            })
        };

        self.fut.replace(Box::pin(self.http.request(request)));

        Ok(())
    }
}

poll_req!(DeleteGuildIntegration<'_>, ());
