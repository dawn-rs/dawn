use super::{InteractionError, InteractionErrorType};
use crate::request::prelude::*;
use twilight_model::id::{ApplicationId, CommandId};

/// Delete a global command, by ID.
pub struct DeleteGlobalCommand<'a> {
    application_id: ApplicationId,
    command_id: CommandId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
}

impl<'a> DeleteGlobalCommand<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: Option<ApplicationId>,
        command_id: CommandId,
    ) -> Result<Self, InteractionError> {
        let application_id = application_id.ok_or(InteractionError{ kind: InteractionErrorType::ApplicationIdNotPresent })?;

        Ok(Self {
            application_id,
            command_id,
            fut: None,
            http,
        })
    }

    fn start(&mut self) -> Result<()> {
        let request = Request::from_route(Route::DeleteGlobalCommand {
            application_id: self.application_id.0,
            command_id: self.command_id.0,
        });
        
        self.fut.replace(Box::pin(self.http.verify(request)));

        Ok(())
    }
}

poll_req!(DeleteGlobalCommand<'_>, ());
