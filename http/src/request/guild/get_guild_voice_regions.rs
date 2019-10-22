use crate::request::prelude::*;
use dawn_model::{id::GuildId, voice::VoiceRegion};

pub struct GetGuildVoiceRegions<'a> {
    fut: Option<Pending<'a, Vec<VoiceRegion>>>,
    guild_id: GuildId,
    http: &'a Client,
}

impl<'a> GetGuildVoiceRegions<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId) -> Self {
        Self {
            fut: None,
            guild_id,
            http,
        }
    }

    fn start(&mut self) -> Result<()> {
        self.fut.replace(Box::pin(self.http.request(Request::from(
            Route::GetGuildVoiceRegions {
                guild_id: self.guild_id.0,
            },
        ))));

        Ok(())
    }
}

poll_req!(GetGuildVoiceRegions<'_>, Vec<VoiceRegion>);
