pub mod add_thread_member;
pub mod create_thread;
pub mod create_thread_from_message;
pub mod get_active_threads;
pub mod get_joined_private_archived_threads;
pub mod get_private_archived_threads;
pub mod get_public_archived_threads;
pub mod get_thread_members;
pub mod join_thread;
pub mod leave_thread;
pub mod remove_thread_member;
pub mod update_thread;

pub use self::{
    add_thread_member::AddThreadMember, create_thread::CreateThread,
    create_thread_from_message::CreateThreadFromMessage, get_active_threads::GetActiveThreads,
    get_joined_private_archived_threads::GetJoinedPrivateArchivedThreads,
    get_private_archived_threads::GetPrivateArchivedThreads,
    get_public_archived_threads::GetPublicArchivedThreads, get_thread_members::GetThreadMembers,
    join_thread::JoinThread, leave_thread::LeaveThread, remove_thread_member::RemoveThreadMember,
    update_thread::UpdateThread,
};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::channel::ChannelType;

/// Returned when the thread can not be updated as configured.
#[derive(Debug)]
pub struct ThreadValidationError {
    kind: ThreadValidationErrorType,
}

impl ThreadValidationError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &ThreadValidationErrorType {
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
        ThreadValidationErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for ThreadValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            ThreadValidationErrorType::NameInvalid { .. } => {
                f.write_str("the length of the name is invalid")
            }
            ThreadValidationErrorType::RateLimitPerUserInvalid { .. } => {
                f.write_str("the rate limit per user is invalid")
            }
            ThreadValidationErrorType::TypeInvalid { kind } => {
                Display::fmt(kind.name(), f)?;

                f.write_str(" is not a thread")
            }
        }
    }
}

impl Error for ThreadValidationError {}

/// Type of error that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum ThreadValidationErrorType {
    /// The length of the name is either fewer than 2 UTF-16 characters or
    /// more than 100 UTF-16 characters.
    NameInvalid {
        /// Provided name.
        name: String,
    },
    /// The seconds of the rate limit per user is more than 21600.
    RateLimitPerUserInvalid {
        /// Provided ratelimit is invalid.
        rate_limit_per_user: u64,
    },
    /// Provided type was not a thread.
    TypeInvalid {
        /// Provided type.
        kind: ChannelType,
    },
}
