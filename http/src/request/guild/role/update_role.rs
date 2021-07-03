use crate::{
    client::Client,
    request::{self, AuditLogReason, AuditLogReasonError, NullableField, Request},
    response::ResponseFuture,
    routing::Route,
};
use serde::Serialize;
use twilight_model::{
    guild::{Permissions, Role},
    id::{GuildId, RoleId},
};

#[derive(Default, Serialize)]
struct UpdateRoleFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<NullableField<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mentionable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<NullableField<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
}

/// Update a role by guild id and its id.
pub struct UpdateRole<'a> {
    fields: UpdateRoleFields,
    guild_id: GuildId,
    http: &'a Client,
    role_id: RoleId,
    reason: Option<String>,
}

impl<'a> UpdateRole<'a> {
    pub(crate) fn new(http: &'a Client, guild_id: GuildId, role_id: RoleId) -> Self {
        Self {
            fields: UpdateRoleFields::default(),
            guild_id,
            http,
            role_id,
            reason: None,
        }
    }

    /// Set the color of the role.
    pub fn color(mut self, color: impl Into<Option<u32>>) -> Self {
        self.fields
            .color
            .replace(NullableField::from_option(color.into()));

        self
    }

    /// If true, display the role in the members list.
    pub fn hoist(mut self, hoist: bool) -> Self {
        self.fields.hoist.replace(hoist);

        self
    }

    /// If true, the role can be @mentioned (pinged) in chat.
    pub fn mentionable(mut self, mentionable: bool) -> Self {
        self.fields.mentionable.replace(mentionable);

        self
    }

    /// Set the name of the role.
    pub fn name(mut self, name: impl Into<Option<String>>) -> Self {
        self.fields
            .name
            .replace(NullableField::from_option(name.into()));

        self
    }

    /// Set the allowed permissions of this role.
    pub fn permissions(mut self, permissions: Permissions) -> Self {
        self.fields.permissions.replace(permissions);

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Role> {
        let mut request = Request::builder(Route::UpdateRole {
            guild_id: self.guild_id.0,
            role_id: self.role_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        if let Some(reason) = &self.reason {
            let header = match request::audit_header(reason) {
                Ok(header) => header,
                Err(source) => return ResponseFuture::error(source),
            };

            request = request.headers(header);
        }

        self.http.request(request.build())
    }
}

impl<'a> AuditLogReason for UpdateRole<'a> {
    fn reason(mut self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError> {
        self.reason
            .replace(AuditLogReasonError::validate(reason.into())?);

        Ok(self)
    }
}
