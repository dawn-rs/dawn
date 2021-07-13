use crate::{
    client::Client,
    error::Error as HttpError,
    request::{
        self, validate_inner, AuditLogReason, AuditLogReasonError, NullableField, Pending, Request,
    },
    routing::Route,
};
use hyper::body::Bytes;
use serde::{de::DeserializeSeed, Serialize};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_model::{
    guild::member::{Member, MemberDeserializer},
    id::{ChannelId, GuildId, RoleId, UserId},
};

#[cfg(not(feature = "simd-json"))]
use serde_json::Value;
#[cfg(feature = "simd-json")]
use simd_json::value::OwnedValue as Value;

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
            UpdateGuildMemberErrorType::NicknameInvalid { .. } => {
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
    NicknameInvalid { nickname: String },
}

#[derive(Default, Serialize)]
struct UpdateGuildMemberFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<NullableField<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<RoleId>>,
}

/// Update a guild member.
///
/// All fields are optional. Refer to [the discord docs] for more information.
///
/// [the discord docs]: https://discord.com/developers/docs/resources/guild#modify-guild-member
pub struct UpdateGuildMember<'a> {
    fields: UpdateGuildMemberFields,
    fut: Option<Pending<'a, Bytes>>,
    guild_id: GuildId,
    http: &'a Client,
    user_id: UserId,
    reason: Option<String>,
}

impl<'a> UpdateGuildMember<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, user_id: UserId) -> Self {
        Self {
            fields: UpdateGuildMemberFields::default(),
            fut: None,
            guild_id,
            http,
            user_id,
            reason: None,
        }
    }

    /// Move the member to a different voice channel.
    pub fn channel_id(mut self, channel_id: impl Into<Option<ChannelId>>) -> Self {
        let channel_id = channel_id.into();
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
    pub fn nick(self, nick: impl Into<Option<String>>) -> Result<Self, UpdateGuildMemberError> {
        self._nick(nick.into())
    }

    fn _nick(mut self, nick: Option<String>) -> Result<Self, UpdateGuildMemberError> {
        if let Some(nick) = nick {
            if !validate_inner::nickname(&nick) {
                return Err(UpdateGuildMemberError {
                    kind: UpdateGuildMemberErrorType::NicknameInvalid { nickname: nick },
                });
            }

            self.fields.nick.replace(NullableField::Value(nick));
        } else {
            self.fields.nick.replace(NullableField::Null);
        }

        Ok(self)
    }

    /// Set the new list of roles for a member.
    pub fn roles(mut self, roles: Vec<RoleId>) -> Self {
        self.fields.roles.replace(roles);

        self
    }

    fn request(&self) -> Result<Request, HttpError> {
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

    fn start(&mut self) -> Result<(), HttpError> {
        let request = self.request()?;
        self.fut.replace(Box::pin(self.http.request_bytes(request)));

        Ok(())
    }
}

impl<'a> AuditLogReason for UpdateGuildMember<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}

impl Future for UpdateGuildMember<'_> {
    type Output = Result<Member, HttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(fut) = self.as_mut().fut.as_mut() {
                let bytes = match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(bytes)) => bytes,
                    Poll::Ready(Err(why)) => return Poll::Ready(Err(why)),
                    Poll::Pending => return Poll::Pending,
                };

                let value = crate::json::from_bytes::<Value>(&bytes).map_err(HttpError::json)?;

                let member_deserializer = MemberDeserializer::new(self.guild_id);
                let member = member_deserializer
                    .deserialize(value)
                    .map_err(HttpError::json)?;

                return Poll::Ready(Ok(member));
            }

            if let Err(why) = self.as_mut().start() {
                return Poll::Ready(Err(why));
            }
        }
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
        let client = Client::new("foo");
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
        assert_eq!(actual.path, expected.path);

        Ok(())
    }

    #[test]
    fn test_nick_set_null() -> Result<(), Box<dyn Error>> {
        let client = Client::new("foo");
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
        let client = Client::new("foo");
        let builder =
            UpdateGuildMember::new(&client, GUILD_ID, USER_ID).nick(Some("foo".to_owned()))?;
        let actual = builder.request()?;

        let body = UpdateGuildMemberFields {
            nick: Some(NullableField::Value("foo".to_owned())),
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
