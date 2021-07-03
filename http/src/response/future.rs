use super::{Response, StatusCode};
use crate::{
    api_error::ApiError,
    error::{Error, ErrorType},
    ratelimiting::RatelimitHeaders,
};
use hyper::{
    body::Bytes, client::ResponseFuture as HyperResponseFuture, StatusCode as HyperStatusCode,
};
use std::{
    convert::TryFrom,
    future::Future,
    marker::PhantomData,
    mem,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    sync::oneshot::{Receiver, Sender},
    time::{self, Timeout},
};
use twilight_model::id::GuildId;

type Output<T> = Result<Response<T>, Error>;

enum InnerPoll<T> {
    Advance(ResponseFutureStage),
    Pending(ResponseFutureStage),
    Ready(Output<T>),
}

struct Chunking {
    future: Pin<Box<dyn Future<Output = Result<Bytes, Error>> + Send + Sync + 'static>>,
    status: HyperStatusCode,
}

impl Chunking {
    fn poll<T>(mut self, cx: &mut Context<'_>) -> InnerPoll<T> {
        let bytes = match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(Ok(bytes)) => bytes,
            Poll::Ready(Err(source)) => return InnerPoll::Ready(Err(source)),
            Poll::Pending => {
                return InnerPoll::Pending(ResponseFutureStage::Chunking(Self {
                    future: self.future,
                    status: self.status,
                }))
            }
        };

        let error = match crate::json::from_bytes::<ApiError>(&bytes) {
            Ok(error) => error,
            Err(source) => {
                return InnerPoll::Ready(Err(Error {
                    kind: ErrorType::Parsing {
                        body: bytes.to_vec(),
                    },
                    source: Some(Box::new(source)),
                }));
            }
        };

        #[cfg(feature = "tracing")]
        if let ApiError::General(ref general) = error {
            use crate::api_error::ErrorCode;

            if let ErrorCode::Other(num) = general.code {
                tracing::debug!("got unknown API error code variant: {}; {:?}", num, error);
            }
        }

        InnerPoll::Ready(Err(Error {
            kind: ErrorType::Response {
                body: bytes.to_vec(),
                error,
                status: StatusCode::new(self.status.as_u16()),
            },
            source: None,
        }))
    }
}

struct Failed {
    source: Error,
}

impl Failed {
    fn poll<T>(self, _: &mut Context<'_>) -> InnerPoll<T> {
        InnerPoll::Ready(Err(self.source))
    }
}

struct InFlight {
    future: Pin<Box<Timeout<HyperResponseFuture>>>,
    guild_id: Option<GuildId>,
    token_invalid: Arc<AtomicBool>,
    tx: Option<Sender<Option<RatelimitHeaders>>>,
}

impl InFlight {
    fn poll<T>(mut self, cx: &mut Context<'_>) -> InnerPoll<T> {
        let resp = match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(Ok(Ok(resp))) => resp,
            Poll::Ready(Ok(Err(source))) => {
                return InnerPoll::Ready(Err(Error {
                    kind: ErrorType::RequestTimedOut,
                    source: Some(Box::new(source)),
                }))
            }
            Poll::Ready(Err(source)) => {
                return InnerPoll::Ready(Err(Error {
                    kind: ErrorType::RequestError,
                    source: Some(Box::new(source)),
                }))
            }
            Poll::Pending => {
                return InnerPoll::Pending(ResponseFutureStage::InFlight(Self {
                    future: self.future,
                    guild_id: self.guild_id,
                    token_invalid: self.token_invalid,
                    tx: self.tx,
                }))
            }
        };

        // If the API sent back an Unauthorized response, then the client's
        // configured token is permanently invalid and future requests must be
        // ignored to avoid API bans.
        if resp.status() == HyperStatusCode::UNAUTHORIZED {
            self.token_invalid.store(true, Ordering::Relaxed);
        }

        if let Some(tx) = self.tx {
            match RatelimitHeaders::try_from(resp.headers()) {
                Ok(v) => {
                    let _res = tx.send(Some(v));
                }
                #[cfg_attr(not(feature = "tracing"), allow(unused_variables))]
                Err(source) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("header parsing failed: {:?}; {:?}", source, resp);

                    let _res = tx.send(None);
                }
            }
        }

        let status = resp.status();

        if status.is_success() {
            let mut response = Response::new(resp);

            if let Some(guild_id) = self.guild_id {
                response.set_guild_id(guild_id);
            }

            return InnerPoll::Ready(Ok(response));
        }

        match status {
            HyperStatusCode::TOO_MANY_REQUESTS => {
                #[cfg(feature = "tracing")]
                tracing::warn!("429 response: {:?}", resp)
            }
            HyperStatusCode::SERVICE_UNAVAILABLE => {
                return InnerPoll::Ready(Err(Error {
                    kind: ErrorType::ServiceUnavailable { response: resp },
                    source: None,
                }));
            }
            _ => {}
        }

        let fut = Box::pin(async {
            hyper::body::to_bytes(resp.into_body())
                .await
                .map_err(|source| Error {
                    kind: ErrorType::ChunkingResponse,
                    source: Some(Box::new(source)),
                })
        });

        InnerPoll::Advance(ResponseFutureStage::Chunking(Chunking {
            future: fut,
            status,
        }))
    }
}

struct RatelimitQueue {
    guild_id: Option<GuildId>,
    request_timeout: Duration,
    response_future: HyperResponseFuture,
    token_invalid: Arc<AtomicBool>,
    wait_for_sender: Receiver<Sender<Option<RatelimitHeaders>>>,
}

impl RatelimitQueue {
    fn poll<T>(mut self, cx: &mut Context<'_>) -> InnerPoll<T> {
        let tx = match Pin::new(&mut self.wait_for_sender).poll(cx) {
            Poll::Ready(Ok(tx)) => tx,
            Poll::Ready(Err(source)) => {
                return InnerPoll::Ready(Err(Error {
                    kind: ErrorType::RequestCanceled {},
                    source: Some(Box::new(source)),
                }))
            }
            Poll::Pending => {
                return InnerPoll::Pending(ResponseFutureStage::RatelimitQueue(Self {
                    guild_id: self.guild_id,
                    request_timeout: self.request_timeout,
                    response_future: self.response_future,
                    token_invalid: self.token_invalid,
                    wait_for_sender: self.wait_for_sender,
                }))
            }
        };

        InnerPoll::Advance(ResponseFutureStage::InFlight(InFlight {
            future: Box::pin(time::timeout(self.request_timeout, self.response_future)),
            guild_id: self.guild_id,
            token_invalid: self.token_invalid,
            tx: Some(tx),
        }))
    }
}

enum ResponseFutureStage {
    Chunking(Chunking),
    Completed,
    Failed(Failed),
    InFlight(InFlight),
    RatelimitQueue(RatelimitQueue),
}

/// Future that will resolve to a [`Response`].
///
/// # Errors
///
///
/// Returns an [`ErrorType::Json`] error type if serializing the response body
/// of the request failed.
///
/// Returns an [`ErrorType::Parsing`] error type if the request failed and the
/// error in the response body could not be deserialized.
///
/// Returns an [`ErrorType::RequestCanceled`] error type if the request was
/// canceled by the user.
///
/// Returns an [`ErrorType::RequestError`] error type if creating the request
/// failed.
///
/// Returns an [`ErrorType::RequestTimedOut`] error type if the request timed
/// out. The timeout value is configured via [`ClientBuilder::timeout`].
///
/// Returns an [`ErrorType::Response`] error type if the request failed.
///
/// Returns an [`ErrorType::ServiceUnavailable`] error type if the Discord API
/// is unavailable.
///
/// [`ClientBuilder::timeout`]: crate::client::ClientBuilder::timeout
/// [`ErrorType::Json`]: crate::error::ErrorType::Json
/// [`ErrorType::Parsing`]: crate::error::ErrorType::Parsing
/// [`ErrorType::RequestCanceled`]: crate::error::ErrorType::RequestCanceled
/// [`ErrorType::RequestError`]: crate::error::ErrorType::RequestError
/// [`ErrorType::RequestTimedOut`]: crate::error::ErrorType::RequestTimedOut
/// [`ErrorType::Response`]: crate::error::ErrorType::Response
/// [`ErrorType::ServiceUnavailable`]: crate::error::ErrorType::ServiceUnavailable
/// [`Response`]: super::Response
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ResponseFuture<T> {
    phantom: PhantomData<T>,
    stage: ResponseFutureStage,
}

impl<T> ResponseFuture<T> {
    pub(crate) fn new(
        token_invalid: Arc<AtomicBool>,
        future: Timeout<HyperResponseFuture>,
        ratelimit_tx: Option<Sender<Option<RatelimitHeaders>>>,
    ) -> Self {
        Self {
            phantom: PhantomData,
            stage: ResponseFutureStage::InFlight(InFlight {
                future: Box::pin(future),
                guild_id: None,
                token_invalid,
                tx: ratelimit_tx,
            }),
        }
    }

    pub(crate) const fn error(source: Error) -> Self {
        Self {
            phantom: PhantomData,
            stage: ResponseFutureStage::Failed(Failed { source }),
        }
    }

    pub(crate) fn ratelimit(
        guild_id: Option<GuildId>,
        token_invalid: Arc<AtomicBool>,
        rx: Receiver<Sender<Option<RatelimitHeaders>>>,
        request_timeout: Duration,
        response_future: HyperResponseFuture,
    ) -> Self {
        Self {
            phantom: PhantomData,
            stage: ResponseFutureStage::RatelimitQueue(RatelimitQueue {
                guild_id,
                response_future,
                wait_for_sender: rx,
                request_timeout,
                token_invalid,
            }),
        }
    }

    /// Set the ID of the relevant guild.
    ///
    /// Necessary for [`MemberBody`] and [`MemberListBody`] deserialization.
    pub(crate) fn set_guild_id(&mut self, guild_id: GuildId) {
        match &mut self.stage {
            ResponseFutureStage::InFlight(ref mut stage) => {
                stage.guild_id.replace(guild_id);
            }
            ResponseFutureStage::RatelimitQueue(ref mut stage) => {
                stage.guild_id.replace(guild_id);
            }
            _ => {}
        }
    }
}

impl<T: Unpin> Future for ResponseFuture<T> {
    type Output = Output<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let stage = mem::replace(&mut self.stage, ResponseFutureStage::Completed);

            let result = match stage {
                ResponseFutureStage::Chunking(chunking) => chunking.poll(cx),
                ResponseFutureStage::Completed => panic!("future already completed"),
                ResponseFutureStage::Failed(failed) => failed.poll(cx),
                ResponseFutureStage::InFlight(in_flight) => in_flight.poll(cx),
                ResponseFutureStage::RatelimitQueue(queue) => queue.poll(cx),
            };

            match result {
                InnerPoll::Advance(stage) => {
                    self.stage = stage;
                }
                InnerPoll::Pending(stage) => {
                    self.stage = stage;

                    return Poll::Pending;
                }
                InnerPoll::Ready(output) => {
                    self.stage = ResponseFutureStage::Completed;

                    return Poll::Ready(output);
                }
            }
        }
    }
}
