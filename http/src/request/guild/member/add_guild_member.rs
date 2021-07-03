use crate::{
    client::Client,
    request::{validate, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    guild::PartialMember,
    id::{GuildId, RoleId, UserId},
};

/// Member cannot be added as configured.
#[derive(Debug)]
pub struct AddGuildMemberError {
    kind: AddGuildMemberErrorType,
}

impl AddGuildMemberError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &AddGuildMemberErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        AddGuildMemberErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for AddGuildMemberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            AddGuildMemberErrorType::NicknameInvalid => f.write_str("nickname length is invalid"),
        }
    }
}

impl Error for AddGuildMemberError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum AddGuildMemberErrorType {
    /// Nickname is either empty or the length is more than 32 UTF-16
    /// characters.
    NicknameInvalid,
}

#[derive(Serialize)]
struct AddGuildMemberFields<'a> {
    pub access_token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<&'a [RoleId]>,
}

pub struct AddGuildMember<'a> {
    fields: AddGuildMemberFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
}

/// Add a user to a guild.
///
/// An access token for the user with `guilds.join` scope is required. All other
/// fields are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#add-guild-member
impl<'a> AddGuildMember<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        guild_id: GuildId,
        user_id: UserId,
        access_token: &'a str,
    ) -> Self {
        Self {
            fields: AddGuildMemberFields {
                access_token,
                deaf: None,
                mute: None,
                nick: None,
                roles: None,
            },
            guild_id,
            http,
            user_id,
        }
    }

    /// Whether the new member will be unable to hear audio when connected to a
    /// voice channel.
    pub const fn deaf(mut self, deaf: bool) -> Self {
        self.fields.deaf = Some(deaf);

        self
    }

    /// Whether the new member will be unable to speak in voice channels.
    pub const fn mute(mut self, mute: bool) -> Self {
        self.fields.mute = Some(mute);

        self
    }

    /// Set the user's initial nickname.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 32 UTF-16
    /// characters.
    ///
    /// # Errors
    ///
    /// Returns an [`AddGuildMemberErrorType::NicknameInvalid`] error type if
    /// the nickname is too short or too long.
    pub fn nick(mut self, nick: &'a str) -> Result<Self, AddGuildMemberError> {
        if !validate::nickname(&nick) {
            return Err(AddGuildMemberError {
                kind: AddGuildMemberErrorType::NicknameInvalid,
            });
        }

        self.fields.nick.replace(nick);

        Ok(self)
    }

    /// List of roles to assign the new member.
    pub const fn roles(mut self, roles: &'a [RoleId]) -> Self {
        self.fields.roles = Some(roles);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<PartialMember> {
        let mut request = Request::builder(Route::AddGuildMember {
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
