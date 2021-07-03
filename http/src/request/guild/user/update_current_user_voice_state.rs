use crate::{
    client::Client,
    request::{NullableField, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use twilight_model::id::{ChannelId, GuildId};

#[derive(Serialize)]
struct UpdateCurrentUserVoiceStateFields<'a> {
    channel_id: ChannelId,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_to_speak_timestamp: Option<NullableField<&'a str>>,
}

/// Update the current user's voice state.
pub struct UpdateCurrentUserVoiceState<'a> {
    fields: UpdateCurrentUserVoiceStateFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> UpdateCurrentUserVoiceState<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, channel_id: ChannelId) -> Self {
        Self {
            fields: UpdateCurrentUserVoiceStateFields {
                channel_id,
                suppress: None,
                request_to_speak_timestamp: None,
            },
            guild_id,
            http,
        }
    }

    /// Set the user's request to speak.
    ///
    /// Set to an empty string to remove an already-present request.
    ///
    /// # Caveats
    ///
    /// - You are able to set `request_to_speak_timestamp` to any present or
    /// future time.
    pub fn request_to_speak_timestamp(mut self, request_to_speak_timestamp: &'a str) -> Self {
        if request_to_speak_timestamp.is_empty() {
            self.fields
                .request_to_speak_timestamp
                .replace(NullableField::Null);
        } else {
            self.fields
                .request_to_speak_timestamp
                .replace(NullableField::Value(request_to_speak_timestamp));
        }

        self
    }

    /// Toggle the user's suppress state.
    ///
    /// # Caveats
    ///
    /// - You must have the `MUTE_MEMBERS` permission to unsuppress yourself.
    /// You can always suppress yourself.
    pub fn suppress(mut self) -> Self {
        self.fields.suppress.replace(true);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::UpdateCurrentUserVoiceState {
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
