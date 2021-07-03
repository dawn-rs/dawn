use std::fmt::{Display, Formatter, Result as FmtResult};

/// Status code of a response.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Create a new status code from a raw status code.
    pub(crate) const fn new(code: u16) -> Self {
        // We don't need to do any checking that the status code is valid since
        // the value always comes from `hyper`'s status code implementation.
        Self(code)
    }

    /// Raw status code value.
    #[must_use = "status code must be used to be useful"]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Whether the status code is informational.
    ///
    /// This is defined as being between `[100, 200)`.
    #[must_use = "whether a status code is informational must be used"]
    pub const fn is_informational(self) -> bool {
        self.in_range(100, 200)
    }

    /// Whether the status code is a success.
    ///
    /// This is defined as being between `[200, 300)`.
    #[must_use = "whether a status code is a success must be used"]
    pub const fn is_success(self) -> bool {
        self.in_range(200, 300)
    }

    /// Whether the status code is a redirection.
    ///
    /// This is defined as being between `[300, 400)`.
    #[must_use = "whether a status code is redirectional must be used"]
    pub const fn is_redirection(self) -> bool {
        self.in_range(300, 400)
    }

    /// Whether the status code is a client error.
    ///
    /// This is defined as being between `[400, 500)`.
    #[must_use = "whether a status code is a client error must be used"]
    pub const fn is_client_error(self) -> bool {
        self.in_range(400, 500)
    }

    /// Whether the status code is a server error.
    ///
    /// This is defined as being between `[500, 600)`.
    #[must_use = "whether a status code is a server error must be used"]
    pub const fn is_server_error(self) -> bool {
        self.in_range(500, 600)
    }

    /// Whether the status code is within a range.
    ///
    /// The range is defined as `[min, max)`.
    const fn in_range(self, min: u16, max: u16) -> bool {
        self.0 >= min && self.0 < max
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::StatusCode;
    use static_assertions::assert_impl_all;
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
    };

    assert_impl_all!(
        StatusCode: Clone,
        Copy,
        Debug,
        Display,
        Eq,
        Hash,
        PartialEq,
        PartialOrd,
        Ord,
        Send,
        Sync
    );

    #[test]
    fn test_ranges() {
        assert!(StatusCode::new(100).is_informational());
        assert!(StatusCode::new(199).is_informational());
        assert!(StatusCode::new(200).is_success());
        assert!(StatusCode::new(299).is_success());
        assert!(StatusCode::new(300).is_redirection());
        assert!(StatusCode::new(399).is_redirection());
        assert!(StatusCode::new(400).is_client_error());
        assert!(StatusCode::new(499).is_client_error());
        assert!(StatusCode::new(500).is_server_error());
        assert!(StatusCode::new(599).is_server_error());
    }
}
