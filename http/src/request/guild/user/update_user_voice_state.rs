use crate::{
    client::Client,
    request::Request,
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use twilight_model::id::{ChannelId, GuildId, UserId};

#[derive(Serialize)]
struct UpdateUserVoiceStateFields {
    channel_id: ChannelId,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppress: Option<bool>,
}

/// Update another user's voice state.
pub struct UpdateUserVoiceState<'a> {
    fields: UpdateUserVoiceStateFields,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> UpdateUserVoiceState<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        guild_id: GuildId,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            fields: UpdateUserVoiceStateFields {
                channel_id,
                suppress: None,
            },
            guild_id,
            http,
            user_id,
        }
    }

    /// Toggle the user's suppress state.
    ///
    /// # Caveats
    ///
    /// - You must have the [`MUTE_MEMBERS`] permission to use this method.
    /// - When unsuppressed, non-bot users will have their
    /// `request_to_speak_timestamp` set to the current time. Bot users will
    /// not.
    /// - When suppressed, the user will have their `request_to_speak_timestamp`
    /// removed.
    ///
    /// [`MUTE_MEMBERS`]: twilight_model::guild::Permissions::MUTE_MEMBERS
    pub const fn suppress(mut self) -> Self {
        self.fields.suppress = Some(true);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        let mut request = Request::builder(Route::UpdateUserVoiceState {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
