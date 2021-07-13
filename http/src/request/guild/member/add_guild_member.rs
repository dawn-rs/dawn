use crate::{
    client::Client,
    error::Error as HttpError,
    request::{validate_inner, PendingOption, Request},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
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
            AddGuildMemberErrorType::NicknameInvalid { .. } => {
                f.write_str("nickname length is invalid")
            }
        }
    }
}

impl Error for AddGuildMemberError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum AddGuildMemberErrorType {
    /// Nickname is either empty or the length is more than 32 UTF-16
    /// characters.
    NicknameInvalid { nickname: String },
}

#[derive(Serialize)]
struct AddGuildMemberFields {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
}

pub struct AddGuildMember<'a> {
    fields: AddGuildMemberFields,
    fut: Option<PendingOption<'a>>,
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
    pub(crate) fn new(
        http: &'a Client,
        guild_id: GuildId,
        user_id: UserId,
        access_token: impl Into<String>,
    ) -> Self {
        Self::_new(http, guild_id, user_id, access_token.into())
    }

    fn _new(http: &'a Client, guild_id: GuildId, user_id: UserId, access_token: String) -> Self {
        Self {
            fields: AddGuildMemberFields {
                access_token,
                deaf: None,
                mute: None,
                nick: None,
                roles: None,
            },
            fut: None,
            guild_id,
            http,
            user_id,
        }
    }

    /// Whether the new member will be unable to hear audio when connected to a
    /// voice channel.
    pub fn deaf(mut self, deaf: bool) -> Self {
        self.fields.deaf.replace(deaf);

        self
    }

    /// Whether the new member will be unable to speak in voice channels.
    pub fn mute(mut self, mute: bool) -> Self {
        self.fields.mute.replace(mute);

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
    pub fn nick(self, nick: impl Into<String>) -> Result<Self, AddGuildMemberError> {
        self._nick(nick.into())
    }

    fn _nick(mut self, nick: String) -> Result<Self, AddGuildMemberError> {
        if !validate_inner::nickname(&nick) {
            return Err(AddGuildMemberError {
                kind: AddGuildMemberErrorType::NicknameInvalid { nickname: nick },
            });
        }

        self.fields.nick.replace(nick);

        Ok(self)
    }

    /// List of roles to assign the new member.
    pub fn roles(mut self, roles: Vec<RoleId>) -> Self {
        self.fields.roles.replace(roles);

        self
    }

    fn start(&mut self) -> Result<(), HttpError> {
        let request = Request::builder(Route::AddGuildMember {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        })
        .json(&self.fields)?
        .build();

        self.fut.replace(Box::pin(self.http.request_bytes(request)));

        Ok(())
    }
}

impl Future for AddGuildMember<'_> {
    type Output = Result<Option<PartialMember>, HttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(fut) = self.as_mut().fut.as_mut() {
                let bytes = match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(bytes)) => bytes,
                    Poll::Ready(Err(why)) => return Poll::Ready(Err(why)),
                    Poll::Pending => return Poll::Pending,
                };

                if bytes.is_empty() {
                    return Poll::Ready(Ok(None));
                }

                return Poll::Ready(crate::json::parse_bytes(&bytes));
            }

            if let Err(why) = self.as_mut().start() {
                return Poll::Ready(Err(why));
            }
        }
    }
}
