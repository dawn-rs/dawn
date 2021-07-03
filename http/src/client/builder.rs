use super::{Client, State};
use crate::ratelimiting::Ratelimiter;
use hyper::header::HeaderMap;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use twilight_model::{channel::message::allowed_mentions::AllowedMentions, id::ApplicationId};

#[derive(Debug)]
/// A builder for [`Client`].
pub struct ClientBuilder {
    pub(crate) application_id: AtomicU64,
    pub(crate) default_allowed_mentions: Option<AllowedMentions>,
    pub(crate) proxy: Option<Box<str>>,
    pub(crate) ratelimiter: Option<Ratelimiter>,
    pub(crate) default_headers: Option<HeaderMap>,
    pub(crate) timeout: Duration,
    pub(crate) token: Option<Box<str>>,
    pub(crate) use_http: bool,
}

impl ClientBuilder {
    /// Create a new builder to create a [`Client`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the [`Client`].
    pub fn build(self) -> Client {
        #[cfg(feature = "rustls-native-roots")]
        let connector = hyper_rustls::HttpsConnector::with_native_roots();
        #[cfg(all(feature = "rustls-webpki-roots", not(feature = "rustls-native-roots")))]
        let connector = hyper_rustls::HttpsConnector::with_webpki_roots();
        #[cfg(all(
            feature = "hyper-tls",
            not(feature = "rustls-native-roots"),
            not(feature = "rustls-webpki-roots")
        ))]
        let connector = hyper_tls::HttpsConnector::new();

        let http = hyper::client::Builder::default().build(connector);

        Client {
            state: Arc::new(State {
                http,
                default_headers: self.default_headers,
                proxy: self.proxy,
                ratelimiter: self.ratelimiter,
                timeout: self.timeout,
                token_invalid: Arc::new(AtomicBool::new(false)),
                token: self.token,
                application_id: self.application_id,
                default_allowed_mentions: self.default_allowed_mentions,
                use_http: self.use_http,
            }),
        }
    }

    /// Set the [`ApplicationId`] used by interaction methods.
    pub fn application_id(self, application_id: ApplicationId) -> Self {
        self.application_id
            .store(application_id.0, Ordering::Relaxed);

        self
    }

    /// Set the default allowed mentions setting to use on all messages sent through the HTTP
    /// client.
    pub fn default_allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.default_allowed_mentions.replace(allowed_mentions);

        self
    }

    /// Set the proxy to use for all HTTP(S) requests.
    ///
    /// **Note** that this isn't currently a traditional proxy, but is for
    /// working with something like [twilight's HTTP proxy server].
    ///
    /// # Examples
    ///
    /// Set the proxy to `twilight_http_proxy.internal`:
    ///
    /// ```rust
    /// use twilight_http::Client;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::builder()
    ///     .proxy("twilight_http_proxy.internal".to_owned(), true)
    ///     .build();
    /// # Ok(()) }
    /// ```
    ///
    /// [twilight's HTTP proxy server]: https://github.com/twilight-rs/http-proxy
    pub fn proxy(mut self, proxy_url: String, use_http: bool) -> Self {
        self.proxy.replace(proxy_url.into_boxed_str());
        self.use_http = use_http;

        self
    }

    /// Set a ratelimiter to use.
    ///
    /// If the argument is `None` then the client's ratelimiter will be skipped
    /// before making a request.
    ///
    /// If this method is not called at all then a default ratelimiter will be
    /// created by [`ClientBuilder::build`].
    #[allow(clippy::missing_const_for_fn)]
    pub fn ratelimiter(mut self, ratelimiter: Option<Ratelimiter>) -> Self {
        self.ratelimiter = ratelimiter;

        self
    }

    /// Set the timeout for HTTP requests.
    ///
    /// The default is 10 seconds.
    pub const fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;

        self
    }

    /// Set a group headers which are sent in every request.
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers.replace(headers);

        self
    }

    /// Set the token to use for HTTP requests.
    pub fn token(mut self, mut token: String) -> Self {
        let is_bot = token.starts_with("Bot ");
        let is_bearer = token.starts_with("Bearer ");

        // Make sure it is either a bot or bearer token, and assume it's a bot
        // token if no prefix is given
        if !is_bot && !is_bearer {
            token.insert_str(0, "Bot ");
        }

        self.token.replace(token.into_boxed_str());

        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            application_id: AtomicU64::default(),
            default_allowed_mentions: None,
            default_headers: None,
            proxy: None,
            ratelimiter: Some(Ratelimiter::new()),
            timeout: Duration::from_secs(10),
            token: None,
            use_http: false,
        }
    }
}
