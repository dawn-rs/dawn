use crate::{
    client::Client,
    request::{self, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    id::GuildId,
    invite::{WelcomeScreen, WelcomeScreenChannel},
};

#[derive(Default, Serialize)]
struct UpdateGuildWelcomeScreenFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "request::slice_is_empty")]
    welcome_channels: &'a [WelcomeScreenChannel],
}

/// Update the guild's welcome screen.
///
/// Requires the [`MANAGE_GUILD`] permission.
///
/// [`MANAGE_GUILD`]: twilight_model::guild::Permissions::MANAGE_GUILD
pub struct UpdateGuildWelcomeScreen<'a> {
    fields: UpdateGuildWelcomeScreenFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> UpdateGuildWelcomeScreen<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildWelcomeScreenFields {
                description: None,
                enabled: None,
                welcome_channels: &[],
            },
            guild_id,
            http,
        }
    }

    /// Set the description of the welcome screen.
    pub const fn description(mut self, description: &'a str) -> Self {
        self.fields.description = Some(description);

        self
    }

    /// Set whether the welcome screen is enabled.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.fields.enabled = Some(enabled);

        self
    }

    /// Set the channels linked in the welcome screen, with associated metadata.
    pub const fn welcome_channels(mut self, welcome_channels: &'a [WelcomeScreenChannel]) -> Self {
        self.fields.welcome_channels = welcome_channels;

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
