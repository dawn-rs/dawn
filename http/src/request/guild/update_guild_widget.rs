use crate::{
    client::Client,
    request::{NullableField, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::GuildWidget,
    id::{ChannelId, GuildId},
};

#[derive(Default, Serialize)]
struct UpdateGuildWidgetFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

/// Modify the guild widget.
pub struct UpdateGuildWidget<'a> {
    fields: UpdateGuildWidgetFields,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> UpdateGuildWidget<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildWidgetFields::default(),
            guild_id,
            http,
        }
    }

    /// Set which channel to display on the widget.
    pub fn channel_id(mut self, channel_id: impl Into<Option<ChannelId>>) -> Self {
        let channel_id = channel_id.into();
        self.fields
            .channel_id
            .replace(NullableField::from_option(channel_id));

        self
    }

    /// Set to true to enable the guild widget.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.fields.enabled.replace(enabled);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<GuildWidget> {
        let mut request = Request::builder(Route::UpdateGuildWidget {
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
