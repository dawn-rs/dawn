use crate::request::prelude::*;
use twilight_model::{
    guild::Ban,
    id::{GuildId, UserId},
};

/// Get information about a ban of a guild.
///
/// Includes the user banned and the reason.
pub struct GetBan<'a> {
    fut: Option<PendingOption<'a>>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

impl<'a> GetBan<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            fut: None,
            guild_id,
            http,
            user_id,
        }
    }

    fn start(&mut self) -> Result<()> {
        self.fut
            .replace(Box::pin(self.http.request_bytes(Request::from(
                Route::GetBan {
                    guild_id: self.guild_id.0,
                    user_id: self.user_id.0,
                },
            ))));

        Ok(())
    }
}

poll_req!(opt, GetBan<'_>, Ban);
