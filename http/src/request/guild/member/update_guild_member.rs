use crate::{
    client::Client,
    error::Error as HttpError,
    request::{self, validate, AuditLogReason, AuditLogReasonError, NullableField, Request},
    response::{marker::MemberBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

/// The error created when the member can not be updated as configured.
#[derive(Debug)]
pub struct UpdateGuildMemberError {
    kind: UpdateGuildMemberErrorType,
}

impl UpdateGuildMemberError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &UpdateGuildMemberErrorType {
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
        UpdateGuildMemberErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for UpdateGuildMemberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            UpdateGuildMemberErrorType::NicknameInvalid => {
                f.write_str("the nickname length is invalid")
            }
        }
    }
}

impl Error for UpdateGuildMemberError {}

/// Type of [`UpdateGuildMemberError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateGuildMemberErrorType {
    /// The nickname is either empty or the length is more than 32 UTF-16 characters.
    NicknameInvalid,
}

#[derive(Default, Serialize)]
struct UpdateGuildMemberFields<'a> {
    #[allow(clippy::option_option)]
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<NullableField<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<&'a [RoleId]>,
}

/// Update a guild member.
///
/// All fields are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild-member
pub struct UpdateGuildMember<'a> {
    fields: UpdateGuildMemberFields<'a>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
    reason: Option<&'a str>,
}

impl<'a> UpdateGuildMember<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            fields: UpdateGuildMemberFields::default(),
            guild_id,
            http,
            user_id,
            reason: None,
        }
    }

    /// Move the member to a different voice channel.
    pub fn channel_id(mut self, channel_id: Option<ChannelId>) -> Self {
        self.fields
            .channel_id
            .replace(NullableField::from_option(channel_id));

        self
    }

    /// If true, restrict the member's ability to hear sound from a voice channel.
    pub fn deaf(mut self, deaf: bool) -> Self {
        self.fields.deaf.replace(deaf);

        self
    }

    /// If true, restrict the member's ability to speak in a voice channel.
    pub fn mute(mut self, mute: bool) -> Self {
        self.fields.mute.replace(mute);

        self
    }

    /// Set the nickname.
    ///
    /// The minimum length is 1 UTF-16 character and the maximum is 32 UTF-16 characters.
    ///
    /// # Errors
    ///
    /// Returns an [`UpdateGuildMemberErrorType::NicknameInvalid`] error type if
    /// the nickname length is too short or too long.
    pub fn nick(mut self, nick: Option<&'a str>) -> Result<Self, UpdateGuildMemberError> {
        if let Some(nick) = nick {
            if !validate::nickname(nick) {
                return Err(UpdateGuildMemberError {
                    kind: UpdateGuildMemberErrorType::NicknameInvalid,
                });
            }

            self.fields.nick.replace(NullableField::Value(nick));
        } else {
            self.fields.nick.replace(NullableField::Null);
        }

        Ok(self)
    }

    /// Set the new list of roles for a member.
    pub fn roles(mut self, roles: &'a [RoleId]) -> Self {
        self.fields.roles.replace(roles);

        self
    }

    fn request(&self) -> Result<Request<'a>, HttpError> {
        let mut request = Request::builder(Route::UpdateMember {
            guild_id: self.guild_id.0,
            user_id: self.user_id.0,
        })
        .json(&self.fields)?;

        if let Some(reason) = &self.reason {
            request = request.headers(request::audit_header(reason)?);
        }

        Ok(request.build())
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<MemberBody> {
        let request = match self.request() {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        let mut future = self.http.request(request);
        future.set_guild_id(self.guild_id);

        future
    }
}

impl<'a> AuditLogReason<'a> for UpdateGuildMember<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{UpdateGuildMember, UpdateGuildMemberFields};
    use crate::{
        request::{NullableField, Request},
        routing::Route,
        Client,
    };
    use std::error::Error;
    use twilight_model::id::{GuildId, UserId};

    const GUILD_ID: GuildId = GuildId(1);
    const USER_ID: UserId = UserId(1);

    #[test]
    fn test_request() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo".to_owned());
        let builder = UpdateGuildMember::new(&client, GUILD_ID, USER_ID)
            .deaf(true)
            .mute(true);
        let actual = builder.request()?;

        let body = UpdateGuildMemberFields {
            deaf: Some(true),
            mute: Some(true),
            ..UpdateGuildMemberFields::default()
        };
        let route = Route::UpdateMember {
            guild_id: GUILD_ID.0,
            user_id: USER_ID.0,
        };
        let expected = Request::builder(route).json(&body)?.build();

        assert_eq!(actual.body, expected.body);
        assert_eq!(actual.route, expected.route);

        Ok(())
    }

    #[test]
    fn test_nick_set_null() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo".to_owned());
        let builder = UpdateGuildMember::new(&client, GUILD_ID, USER_ID).nick(None)?;
        let actual = builder.request()?;

        let body = UpdateGuildMemberFields {
            nick: Some(NullableField::Null),
            ..UpdateGuildMemberFields::default()
        };
        let route = Route::UpdateMember {
            guild_id: GUILD_ID.0,
            user_id: USER_ID.0,
        };
        let expected = Request::builder(route).json(&body)?.build();

        assert_eq!(actual.body, expected.body);

        Ok(())
    }

    #[test]
    fn test_nick_set_value() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo".to_owned());
        let builder = UpdateGuildMember::new(&client, GUILD_ID, USER_ID).nick(Some("foo"))?;
        let actual = builder.request()?;

        let body = UpdateGuildMemberFields {
            nick: Some(NullableField::Value("foo")),
            ..UpdateGuildMemberFields::default()
        };
        let route = Route::UpdateMember {
            guild_id: GUILD_ID.0,
            user_id: USER_ID.0,
        };
        let expected = Request::builder(route).json(&body)?.build();

        assert_eq!(actual.body, expected.body);

        Ok(())
    }
}
