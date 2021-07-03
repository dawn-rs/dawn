use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::Emoji,
    id::{EmojiId, GuildId, RoleId},
};

#[derive(Serialize)]
struct UpdateEmojiFields<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<&'a [RoleId]>,
}

/// Update an emoji in a guild, by id.
pub struct UpdateEmoji<'a> {
    emoji_id: EmojiId,
    fields: UpdateEmojiFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
    reason: Option<&'a str>,
}

impl<'a> UpdateEmoji<'a> {
    pub(crate) const fn new(http: &'a Client, guild_id: GuildId, emoji_id: EmojiId) -> Self {
        Self {
            fields: UpdateEmojiFields {
                name: None,
                roles: None,
            },
            emoji_id,
            guild_id,
            http,
            reason: None,
        }
    }

    /// Change the name of the emoji.
    pub const fn name(mut self, name: &'a str) -> Self {
        self.fields.name = Some(name);

        self
    }

    /// Change the roles that the emoji is whitelisted to.
    pub const fn roles(mut self, roles: &'a [RoleId]) -> Self {
        self.fields.roles = Some(roles);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Emoji> {
        let mut request = Request::builder(Route::UpdateEmoji {
            emoji_id: self.emoji_id.0,
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        if let Some(reason) = self.reason.as_ref() {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason<'a> for UpdateEmoji<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}
