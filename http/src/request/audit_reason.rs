use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub trait AuditLogReason: private::Sealed {
    fn reason(self, reason: impl Into<String>) -> Result<Self, AuditLogReasonError>
    where
        Self: Sized;
}

mod private {
    use crate::request::{
        channel::{
            invite::{CreateInvite, DeleteInvite},
            message::{DeleteMessage, DeleteMessages},
            thread::UpdateThread,
            webhook::{
                CreateWebhook, DeleteWebhook, DeleteWebhookMessage, UpdateWebhook,
                UpdateWebhookMessage,
            },
            CreatePin, DeleteChannel, DeleteChannelPermissionConfigured, DeletePin, UpdateChannel,
            UpdateChannelPermissionConfigured,
        },
        guild::{
            ban::{CreateBan, DeleteBan},
            emoji::{CreateEmoji, DeleteEmoji, UpdateEmoji},
            integration::DeleteGuildIntegration,
            member::{AddRoleToMember, RemoveMember, RemoveRoleFromMember, UpdateGuildMember},
            role::{CreateRole, DeleteRole, UpdateRole},
            CreateGuildChannel, CreateGuildPrune, UpdateGuild,
        },
    };

    /// Sealed stops crates other crates implementing the trait.
    pub trait Sealed {}
    impl<'a> Sealed for CreateInvite<'a> {}
    impl<'a> Sealed for DeleteInvite<'a> {}
    impl<'a> Sealed for DeleteMessage<'a> {}
    impl<'a> Sealed for DeleteMessages<'a> {}
    impl<'a> Sealed for UpdateChannel<'a> {}
    impl<'a> Sealed for CreateWebhook<'a> {}
    impl Sealed for DeleteWebhookMessage<'_> {}
    impl<'a> Sealed for DeleteWebhook<'a> {}
    impl<'a> Sealed for UpdateWebhook<'a> {}
    impl<'a> Sealed for CreatePin<'a> {}
    impl<'a> Sealed for DeleteChannel<'a> {}
    impl<'a> Sealed for DeleteChannelPermissionConfigured<'a> {}
    impl<'a> Sealed for DeletePin<'a> {}
    impl<'a> Sealed for UpdateChannelPermissionConfigured<'a> {}
    impl<'a> Sealed for CreateBan<'a> {}
    impl<'a> Sealed for DeleteBan<'a> {}
    impl<'a> Sealed for CreateGuildChannel<'a> {}
    impl<'a> Sealed for CreateGuildPrune<'a> {}
    impl<'a> Sealed for CreateEmoji<'a> {}
    impl<'a> Sealed for DeleteEmoji<'a> {}
    impl<'a> Sealed for UpdateEmoji<'a> {}
    impl<'a> Sealed for DeleteGuildIntegration<'a> {}
    impl<'a> Sealed for UpdateGuildMember<'a> {}
    impl<'a> Sealed for AddRoleToMember<'a> {}
    impl<'a> Sealed for RemoveMember<'a> {}
    impl<'a> Sealed for RemoveRoleFromMember<'a> {}
    impl<'a> Sealed for CreateRole<'a> {}
    impl<'a> Sealed for DeleteRole<'a> {}
    impl<'a> Sealed for UpdateRole<'a> {}
    impl<'a> Sealed for UpdateGuild<'a> {}
    impl<'a> Sealed for UpdateThread<'a> {}
    impl Sealed for UpdateWebhookMessage<'_> {}
}

impl AuditLogReasonError {
    /// The maximum audit log reason length in UTF-16 codepoints.
    pub const AUDIT_REASON_LENGTH: usize = 512;

    pub(crate) fn validate(reason: String) -> Result<String, AuditLogReasonError> {
        if reason.chars().count() <= Self::AUDIT_REASON_LENGTH {
            Ok(reason)
        } else {
            Err(AuditLogReasonError {
                kind: AuditLogReasonErrorType::TooLarge { reason },
            })
        }
    }
}

/// The error created when a reason can not be used as configured.
#[derive(Debug)]
pub struct AuditLogReasonError {
    kind: AuditLogReasonErrorType,
}

impl AuditLogReasonError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &AuditLogReasonErrorType {
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
        AuditLogReasonErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for AuditLogReasonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            AuditLogReasonErrorType::TooLarge { reason } => {
                f.write_str("the audit log reason is ")?;
                Display::fmt(&reason.chars().count(), f)?;
                f.write_str(" characters long, but the max is ")?;

                Display::fmt(&Self::AUDIT_REASON_LENGTH, f)
            }
        }
    }
}

impl Error for AuditLogReasonError {}

/// Type of [`AuditLogReasonError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum AuditLogReasonErrorType {
    /// Returned when the reason is over 512 UTF-16 characters.
    TooLarge { reason: String },
}

#[cfg(test)]
mod test {
    use super::AuditLogReason;
    use crate::request::{
        channel::{
            invite::{CreateInvite, DeleteInvite},
            message::{DeleteMessage, DeleteMessages},
            webhook::{CreateWebhook, DeleteWebhook, UpdateWebhook},
            CreatePin, DeleteChannel, DeleteChannelPermissionConfigured, DeletePin, UpdateChannel,
            UpdateChannelPermissionConfigured,
        },
        guild::{
            ban::{CreateBan, DeleteBan},
            emoji::{CreateEmoji, DeleteEmoji, UpdateEmoji},
            integration::DeleteGuildIntegration,
            member::{AddRoleToMember, RemoveMember, RemoveRoleFromMember, UpdateGuildMember},
            role::{CreateRole, DeleteRole, UpdateRole},
            CreateGuildChannel, CreateGuildPrune, UpdateGuild,
        },
    };
    use static_assertions::{assert_impl_all, assert_obj_safe};

    assert_obj_safe!(AuditLogReason);

    assert_impl_all!(CreateInvite<'_>: AuditLogReason);
    assert_impl_all!(DeleteInvite<'_>: AuditLogReason);
    assert_impl_all!(DeleteMessage<'_>: AuditLogReason);
    assert_impl_all!(DeleteMessages<'_>: AuditLogReason);
    assert_impl_all!(UpdateChannel<'_>: AuditLogReason);
    assert_impl_all!(CreateWebhook<'_>: AuditLogReason);
    assert_impl_all!(DeleteWebhook<'_>: AuditLogReason);
    assert_impl_all!(UpdateWebhook<'_>: AuditLogReason);
    assert_impl_all!(CreatePin<'_>: AuditLogReason);
    assert_impl_all!(DeleteChannel<'_>: AuditLogReason);
    assert_impl_all!(DeleteChannelPermissionConfigured<'_>: AuditLogReason);
    assert_impl_all!(DeletePin<'_>: AuditLogReason);
    assert_impl_all!(UpdateChannelPermissionConfigured<'_>: AuditLogReason);
    assert_impl_all!(CreateBan<'_>: AuditLogReason);
    assert_impl_all!(DeleteBan<'_>: AuditLogReason);
    assert_impl_all!(CreateGuildChannel<'_>: AuditLogReason);
    assert_impl_all!(CreateGuildPrune<'_>: AuditLogReason);
    assert_impl_all!(CreateEmoji<'_>: AuditLogReason);
    assert_impl_all!(DeleteEmoji<'_>: AuditLogReason);
    assert_impl_all!(UpdateEmoji<'_>: AuditLogReason);
    assert_impl_all!(DeleteGuildIntegration<'_>: AuditLogReason);
    assert_impl_all!(UpdateGuildMember<'_>: AuditLogReason);
    assert_impl_all!(AddRoleToMember<'_>: AuditLogReason);
    assert_impl_all!(RemoveMember<'_>: AuditLogReason);
    assert_impl_all!(RemoveRoleFromMember<'_>: AuditLogReason);
    assert_impl_all!(CreateRole<'_>: AuditLogReason);
    assert_impl_all!(DeleteRole<'_>: AuditLogReason);
    assert_impl_all!(UpdateRole<'_>: AuditLogReason);
    assert_impl_all!(UpdateGuild<'_>: AuditLogReason);
}
