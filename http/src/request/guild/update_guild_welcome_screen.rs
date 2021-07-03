use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use serde::Serialize;
use twilight_model::{
    id::GuildId,
    invite::{WelcomeScreen, WelcomeScreenChannel},
};

#[derive(Default, Serialize)]
struct UpdateGuildWelcomeScreenFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    welcome_channels: Vec<WelcomeScreenChannel>,
}

/// Update the guild's welcome screen.
///
/// Requires the [`MANAGE_GUILD`] permission.
///
/// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
pub struct UpdateGuildWelcomeScreen<'a> {
    fields: UpdateGuildWelcomeScreenFields,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> UpdateGuildWelcomeScreen<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildWelcomeScreenFields::default(),
            guild_id,
            http,
        }
    }

    /// Set the description of the welcome screen.
    pub fn description(self, description: impl Into<String>) -> Self {
        self._description(description.into())
    }

    fn _description(mut self, description: String) -> Self {
        self.fields.description.replace(description);

        self
    }

    /// Set whether the welcome screen is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.fields.enabled.replace(enabled);

        self
    }

    /// Set the channels linked in the welcome screen, with associated metadata.
    pub fn welcome_channels(
        mut self,
        welcome_channels: impl IntoIterator<Item = WelcomeScreenChannel>,
    ) -> Self {
        self.fields.welcome_channels = welcome_channels.into_iter().collect();

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<WelcomeScreen> {
        let mut request = Request::builder(Route::UpdateGuildWelcomeScreen {
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
