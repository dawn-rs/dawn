use crate::request::prelude::*;
use dawn_model::{id::GuildId, invite::Invite};

pub struct GetGuildInvites<'a> {
    fut: Option<Pending<'a, Vec<Invite>>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildInvites<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fut: None,
            guild_id,
            http,
        }
    }

    fn start(&mut self) -> Result<()> {
        self.fut.replace(Box::pin(self.http.request(Request::from(
            Route::GetGuildInvites {
                guild_id: self.guild_id.0,
            },
        ))));

        Ok(())
    }
}

poll_req!(GetGuildInvites<'_>, Vec<Invite>);
