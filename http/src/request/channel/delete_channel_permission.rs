use super::DeleteChannelPermissionConfigured;
use crate::client::Client;
use twilight_model::id::{ChannelId, RoleId, UserId};

/// Clear the permissions for a target ID in a channel.
///
/// The target ID must be set with one of the associated methods.
pub struct DeleteChannelPermission<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> DeleteChannelPermission<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Delete an override for an member.
    pub fn member(self, user_id: impl Into<UserId>) -> DeleteChannelPermissionConfigured<'a> {
        self.configure(user_id.into().0)
    }

    /// Delete an override for an role.
    pub fn role(self, role_id: impl Into<RoleId>) -> DeleteChannelPermissionConfigured<'a> {
        self.configure(role_id.into().0)
    }

    fn configure(self, target_id: u64) -> DeleteChannelPermissionConfigured<'a> {
        DeleteChannelPermissionConfigured::new(self.http, self.channel_id, target_id)
    }
}
