use crate::channel::{thread::ThreadMember, Channel};
use serde::{Deserialize, Serialize};

/// Response body returned in thread listing methods.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ThreadsListing {
    /// Whether there are potentially more threads that could be returned.
    pub has_more: bool,
    /// A thread member object for each returned thread the current user has joined.
    pub members: Vec<ThreadMember>,
    /// List of threads.
    pub threads: Vec<Channel>,
}
