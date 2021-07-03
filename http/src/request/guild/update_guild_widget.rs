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

#[derive(Serialize)]
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
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fields: UpdateGuildWidgetFields {
                channel_id: None,
                enabled: None,
            },
            guild_id,
            http,
        }
    }

    /// Set which channel to display on the widget.
    pub const fn channel_id(mut self, channel_id: Option<ChannelId>) -> Self {
        self.fields.channel_id = Some(NullableField(channel_id));

        self
    }

    /// Set to true to enable the guild widget.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.fields.enabled = Some(enabled);

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
