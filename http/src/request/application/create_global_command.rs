use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        application::{InteractionError, InteractionErrorType},
        validate_inner, Pending, Request,
    },
    routing::Route,
};
use twilight_model::{
    application::command::{Command, CommandOption},
    id::ApplicationId,
};

/// Create a new global command.
///
/// The name must be between 3 and 32 characters in length, and the description
/// must be between 1 and 100 characters in length. Creating a command with the
/// same name as an already-existing global command will overwwrite the old
/// command. See [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#create-global-application-command
pub struct CreateGlobalCommand<'a> {
    command: Command,
    application_id: ApplicationId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
    optional_option_added: bool,
}

impl<'a> CreateGlobalCommand<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, InteractionError> {
        let name = name.into();
        let description = description.into();

        if !validate_inner::command_name(&name) {
            return Err(InteractionError {
                kind: InteractionErrorType::CommandNameValidationFailed { name },
            });
        }
        if !validate_inner::command_description(&description) {
            return Err(InteractionError {
                kind: InteractionErrorType::CommandDescriptionValidationFailed { description },
            });
        }

        Ok(Self {
            command: Command {
                application_id: Some(application_id),
                guild_id: None,
                name,
                default_permission: None,
                description,
                id: None,
                options: vec![],
            },
            application_id,
            fut: None,
            http,
            optional_option_added: false,
        })
    }

    /// Add a command option.
    ///
    /// Required command options must be added before optional options.
    ///
    /// Errors
    ///
    /// Retuns an [`InteractionErrorType::CommandOptionsRequiredFirst`]
    /// if a required option was added after an optional option.
    pub fn add_command_option(mut self, option: CommandOption) -> Result<Self, InteractionError> {
        if !self.optional_option_added && !option.is_required() {
            self.optional_option_added = true
        }

        if option.is_required() && self.optional_option_added {
            return Err(InteractionError {
                kind: InteractionErrorType::CommandOptionsRequiredFirst { option },
            });
        }

        self.command.options.push(option);

        Ok(self)
    }

    /// Whether the command is enabled by default when the app is added to a guild.
    pub fn default_permission(mut self, default: bool) -> Self {
        self.command.default_permission.replace(default);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::CreateGlobalCommand {
            application_id: self.application_id.0,
        })
        .json(&self.command)?;

        self.fut
            .replace(Box::pin(self.http.verify(request.build())));

        Ok(())
    }
}

poll_req!(CreateGlobalCommand<'_>, ());
