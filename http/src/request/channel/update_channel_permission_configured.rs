use crate::{
    client::Client,
    error::Error,
    request::{self, AuditLogReason, AuditLogReasonError, Request},
    response::{marker::EmptyBody, ResponseFuture},
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    channel::permission_overwrite::{PermissionOverwriteTargetType, PermissionOverwriteType},
    guild::Permissions,
    id::ChannelId,
};

#[derive(Serialize)]
struct UpdateChannelPermissionConfiguredFields {
    allow: Permissions,
    deny: Permissions,
    #[serde(rename = "type")]
    kind: PermissionOverwriteTargetType,
}

/// Created when either `member` or `role` is called on a `DeleteChannelPermission` struct.
pub struct UpdateChannelPermissionConfigured<'a> {
    channel_id: ChannelId,
    fields: UpdateChannelPermissionConfiguredFields,
    http: &'a Client,
    target_id: u64,
    reason: Option<&'a str>,
}

impl<'a> UpdateChannelPermissionConfigured<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        allow: Permissions,
        deny: Permissions,
        target: PermissionOverwriteType,
    ) -> Self {
        let (name, target_id) = match target {
            PermissionOverwriteType::Member(user_id) => {
                (PermissionOverwriteTargetType::Member, user_id.0)
            }
            PermissionOverwriteType::Role(role_id) => {
                (PermissionOverwriteTargetType::Role, role_id.0)
            }
        };

        Self {
            channel_id,
            fields: UpdateChannelPermissionConfiguredFields {
                allow,
                deny,
                kind: name,
            },
            http,
            target_id,
            reason: None,
        }
    }

    fn request(&self) -> Result<Request<'a>, Error> {
        let mut request = Request::builder(Route::UpdatePermissionOverwrite {
            channel_id: self.channel_id.0,
            target_id: self.target_id,
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
    pub fn exec(self) -> ResponseFuture<EmptyBody> {
        match self.request() {
            Ok(request) => self.http.request(request),
            Err(source) => ResponseFuture::error(source),
        }
    }
}

impl<'a> AuditLogReason<'a> for UpdateChannelPermissionConfigured<'a> {
    fn reason(mut self, reason: &'a str) -> Result<Self, AuditLogReasonError> {
        self.reason.replace(AuditLogReasonError::validate(reason)?);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{UpdateChannelPermissionConfigured, UpdateChannelPermissionConfiguredFields};
    use crate::{request::Request, routing::Route, Client};
    use twilight_model::{
        channel::permission_overwrite::{PermissionOverwriteTargetType, PermissionOverwriteType},
        guild::Permissions,
        id::{ChannelId, UserId},
    };

    #[test]
    fn test_request() {
        let client = Client::new("foo".to_owned());
        let builder = UpdateChannelPermissionConfigured::new(
            &client,
            ChannelId(1),
            Permissions::empty(),
            Permissions::SEND_MESSAGES,
            PermissionOverwriteType::Member(UserId(2)),
        );
        let actual = builder.request().expect("failed to create request");

        let body = crate::json::to_vec(&UpdateChannelPermissionConfiguredFields {
            allow: Permissions::empty(),
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteTargetType::Member,
        })
        .expect("failed to serialize payload");
        let route = Route::UpdatePermissionOverwrite {
            channel_id: 1,
            target_id: 2,
        };
        let expected = Request::builder(route).body(body).build();

        assert_eq!(expected.body, actual.body);
        assert_eq!(expected.route, actual.route);
    }
}
