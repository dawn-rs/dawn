use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        application::{InteractionError, InteractionErrorType},
        validate, Request, RequestBuilder,
    },
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{
    application::command::{Command, CommandOption},
    id::{ApplicationId, GuildId},
};

/// Create a new command in a guild.
///
/// The name must be between 3 and 32 characters in length, and the description
/// must be between 1 and 100 characters in length. Creating a guild command
/// with the same name as an already-existing guild command in the same guild
/// will overwrite the old command. See [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/interactions/slash-commands#create-guild-application-command
pub struct CreateGuildCommand<'a> {
    application_id: ApplicationId,
    command: Command,
    guild_id: GuildId,
    http: &'a Client,
    optional_option_added: bool,
}

impl<'a> CreateGuildCommand<'a> {
    pub(crate) fn new(
        http: &'a Client,
        application_id: ApplicationId,
        guild_id: GuildId,
        name: String,
        description: String,
    ) -> Result<Self, InteractionError> {
        if !validate::command_name(&name) {
            return Err(InteractionError {
                kind: InteractionErrorType::CommandNameValidationFailed { name },
            });
        }

        if !validate::command_description(&description) {
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
                options: Vec::new(),
            },
            application_id,
            guild_id,
            http,
            optional_option_added: false,
        })
    }

    /// Whether the command is enabled by default when the app is added to
    /// a guild.
    pub fn default_permission(mut self, default: bool) -> Self {
        self.command.default_permission.replace(default);

        self
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

    fn request(&self) -> Result<Request<'a>, HttpError> {
        Request::builder(Route::CreateGuildCommand {
            application_id: self.application_id.0,
            guild_id: self.guild_id.0,
        })
        .json(&self.command)
        .map(RequestBuilder::build)
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
