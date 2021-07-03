use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::invite::Invite;

#[derive(Default)]
struct GetInviteFields {
    with_counts: bool,
    with_expiration: bool,
}

/// Get information about an invite by its code.
///
/// If [`with_counts`] is called, the returned invite will contain approximate
/// member counts. If [`with_expiration`] is called, it will contain the
/// expiration date.
///
/// # Examples
///
/// ```rust,no_run
/// use twilight_http::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("my token");
///
/// let invite = client
///     .invite("code")
///     .with_counts()
///     .exec()
///     .await?;
/// # Ok(()) }
/// ```
///
/// [`with_counts`]: Self::with_counts
/// [`with_expiration`]: Self::with_expiration
pub struct GetInvite<'a> {
    code: String,
    fields: GetInviteFields,
    http: &'a Client,
}

impl<'a> GetInvite<'a> {
    pub(crate) fn new(http: &'a Client, code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            fields: GetInviteFields::default(),
            http,
        }
    }

    /// Whether the invite returned should contain approximate member counts.
    pub const fn with_counts(mut self) -> Self {
        self.fields.with_counts = true;

        self
    }

    /// Whether the invite returned should contain its expiration date.
    pub const fn with_expiration(mut self) -> Self {
        self.fields.with_expiration = true;

        self
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<Invite> {
        let request = Request::from_route(Route::GetInviteWithExpiration {
            code: self.code,
            with_counts: self.fields.with_counts,
            with_expiration: self.fields.with_expiration,
        });

        self.http.request(request)
    }
}
