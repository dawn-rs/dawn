use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    time::Duration,
};
use tokio::{
    sync::Mutex,
    time::{self, Instant},
};

/// Creating a day limiter queue failed.
#[derive(Debug)]
pub struct DayLimiterError {
    kind: DayLimiterErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl Display for DayLimiterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            DayLimiterErrorType::RetrievingSessionAvailability { .. } => {
                f.write_str("retrieving the bot's gateway session availability failed")
            }
        }
    }
}

impl Error for DayLimiterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`DayLimiterError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum DayLimiterErrorType {
    /// Retrieving the bot's available gateway session initiation information
    /// via the HTTP API failed.
    RetrievingSessionAvailability,
}

#[derive(Debug)]
pub(crate) struct DayLimiter(pub(crate) Mutex<DayLimiterInner>);

#[derive(Debug)]
pub(crate) struct DayLimiterInner {
    pub http: twilight_http::Client,
    pub last_check: Instant,
    pub next_reset: Duration,
    pub total: u64,
    pub current: u64,
}

impl DayLimiter {
    pub async fn new(http: &twilight_http::Client) -> Result<Self, DayLimiterError> {
        let info = http
            .gateway()
            .authed()
            .exec()
            .await
            .map_err(|source| DayLimiterError {
                kind: DayLimiterErrorType::RetrievingSessionAvailability,
                source: Some(Box::new(source)),
            })?
            .model()
            .await
            .map_err(|source| DayLimiterError {
                kind: DayLimiterErrorType::RetrievingSessionAvailability,
                source: Some(Box::new(source)),
            })?;

        let last_check = Instant::now();

        let next_reset = Duration::from_millis(info.session_start_limit.reset_after);
        let total = info.session_start_limit.total;
        let remaining = info.session_start_limit.remaining;
        debug_assert!(total >= remaining);
        let current = total - remaining;
        Ok(DayLimiter(Mutex::new(DayLimiterInner {
            http: http.clone(),
            last_check,
            next_reset,
            total: info.session_start_limit.total,
            current,
        })))
    }

    pub async fn get(&self) {
        let mut lock = self.0.lock().await;
        if lock.current < lock.total {
            lock.current += 1;
        } else {
            let wait = lock.last_check + lock.next_reset;
            time::sleep_until(wait).await;
            if let Ok(res) = lock.http.gateway().authed().exec().await {
                if let Ok(info) = res.model().await {
                    let last_check = Instant::now();
                    let next_reset = Duration::from_millis(info.session_start_limit.remaining);
                    tracing::info!("next session start limit reset in: {:.2?}", next_reset);
                    let total = info.session_start_limit.total;
                    let remaining = info.session_start_limit.remaining;
                    assert!(total >= remaining);
                    let current = total - remaining;
                    lock.last_check = last_check;
                    lock.next_reset = next_reset;
                    lock.total = total;
                    lock.current = current + 1;

                    return;
                }
            }

            tracing::warn!(
                "unable to get new session limits, skipping (this may cause bad things)"
            );
        }
    }
}
