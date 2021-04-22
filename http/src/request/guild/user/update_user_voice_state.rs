use crate::request::prelude::*;
use twilight_model::id::{ChannelId, GuildId, UserId};

#[derive(Serialize)]
struct UpdateUserVoiceStateFields {
    channel_id: ChannelId,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppress: Option<bool>,
}

/// Update another user's voice state.
///
/// # Caveats
///
/// - `channel_id` must currently point to a stage channel.
/// - User must already have joined `channel_id`.
pub struct UpdateUserVoiceState<'a> {
    fields: UpdateUserVoiceStateFields,
    fut: Option<Pending<'a, ()>>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> UpdateUserVoiceState<'a> {
    pub(crate) fn new(
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
            fut: None,
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
    /// [`MUTE_MEMBERS`]: twilight_model::guild::permissions::MUTE_MEMBERS
    pub fn suppress(mut self) -> Self {
        self.fields.suppress.replace(true);

        self
    }

    fn start(&mut self) -> Result<()> {
        self.fut.replace(Box::pin(self.http.verify(Request::from((
            crate::json_to_vec(&self.fields)?,
            Route::UpdateUserVoiceState {
                guild_id: self.guild_id.0,
                user_id: self.user_id.0,
            },
        )))));

        Ok(())
    }
}

poll_req!(UpdateUserVoiceState<'_>, ());