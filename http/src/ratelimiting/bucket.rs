use super::{headers::RatelimitHeaders, GlobalLockPair};
use crate::routing::Path;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
        Mutex as AsyncMutex,
    },
    time::{sleep, timeout},
};

#[derive(Clone, Debug)]
pub enum TimeRemaining {
    Finished,
    NotStarted,
    Some(Duration),
}

#[derive(Debug)]
pub struct Bucket {
    pub limit: AtomicU64,
    pub path: Path,
    pub queue: BucketQueue,
    pub remaining: AtomicU64,
    pub reset_after: AtomicU64,
    pub started_at: Mutex<Option<Instant>>,
}

impl Bucket {
    pub fn new(path: Path) -> Self {
        Self {
            limit: AtomicU64::new(u64::max_value()),
            path,
            queue: BucketQueue::default(),
            remaining: AtomicU64::new(u64::max_value()),
            reset_after: AtomicU64::new(u64::max_value()),
            started_at: Mutex::new(None),
        }
    }

    pub fn limit(&self) -> u64 {
        self.limit.load(Ordering::Relaxed)
    }

    pub fn remaining(&self) -> u64 {
        self.remaining.load(Ordering::Relaxed)
    }

    pub fn reset_after(&self) -> u64 {
        self.reset_after.load(Ordering::Relaxed)
    }

    pub fn time_remaining(&self) -> TimeRemaining {
        let reset_after = self.reset_after();
        let started_at = match *self.started_at.lock().expect("bucket poisoned") {
            Some(v) => v,
            None => return TimeRemaining::NotStarted,
        };
        let elapsed = started_at.elapsed();

        if elapsed > Duration::from_millis(reset_after) {
            return TimeRemaining::Finished;
        }

        TimeRemaining::Some(Duration::from_millis(reset_after) - elapsed)
    }

    pub fn try_reset(&self) -> bool {
        if self.started_at.lock().expect("bucket poisoned").is_none() {
            return false;
        }

        if let TimeRemaining::Finished = self.time_remaining() {
            self.remaining.store(self.limit(), Ordering::Relaxed);
            *self.started_at.lock().expect("bucket poisoned") = None;

            true
        } else {
            false
        }
    }

    pub fn update(&self, ratelimits: Option<(u64, u64, u64)>) {
        let bucket_limit = self.limit();

        {
            let mut started_at = self.started_at.lock().expect("bucket poisoned");

            if started_at.is_none() {
                started_at.replace(Instant::now());
            }
        }

        if let Some((limit, remaining, reset_after)) = ratelimits {
            if bucket_limit != limit && bucket_limit == u64::max_value() {
                self.reset_after.store(reset_after, Ordering::SeqCst);
                self.limit.store(limit, Ordering::SeqCst);
            }

            self.remaining.store(remaining, Ordering::Relaxed);
        } else {
            self.remaining.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug)]
pub struct BucketQueue {
    rx: AsyncMutex<UnboundedReceiver<Sender<Sender<Option<RatelimitHeaders>>>>>,
    tx: UnboundedSender<Sender<Sender<Option<RatelimitHeaders>>>>,
}

impl BucketQueue {
    pub fn push(&self, tx: Sender<Sender<Option<RatelimitHeaders>>>) {
        let _sent = self.tx.send(tx);
    }

    pub async fn pop(
        &self,
        timeout_duration: Duration,
    ) -> Option<Sender<Sender<Option<RatelimitHeaders>>>> {
        let mut rx = self.rx.lock().await;

        match timeout(timeout_duration, rx.recv()).await.ok() {
            Some(x) => x,
            None => None,
        }
    }
}

impl Default for BucketQueue {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            rx: AsyncMutex::new(rx),
            tx,
        }
    }
}

pub(super) struct BucketQueueTask {
    bucket: Arc<Bucket>,
    buckets: Arc<Mutex<HashMap<Path, Arc<Bucket>>>>,
    global: Arc<GlobalLockPair>,
    path: Path,
}

impl BucketQueueTask {
    const WAIT: Duration = Duration::from_secs(10);

    pub fn new(
        bucket: Arc<Bucket>,
        buckets: Arc<Mutex<HashMap<Path, Arc<Bucket>>>>,
        global: Arc<GlobalLockPair>,
        path: Path,
    ) -> Self {
        Self {
            bucket,
            buckets,
            global,
            path,
        }
    }

    pub async fn run(self) {
        #[cfg(feature = "tracing")]
        let span = tracing::debug_span!("background queue task", path=?self.path);

        while let Some(queue_tx) = self.next().await {
            let (tx, rx) = oneshot::channel();

            if self.global.is_locked() {
                self.global.0.lock().await;
            }

            let _sent = queue_tx.send(tx);

            #[cfg(feature = "tracing")]
            tracing::debug!(parent: &span, "starting to wait for response headers",);

            // TODO: Find a better way of handling nested types.
            #[allow(clippy::unnested_or_patterns)]
            match timeout(Self::WAIT, rx).await {
                Ok(Ok(Some(headers))) => self.handle_headers(&headers).await,
                // - None was sent through the channel (request aborted)
                // - channel was closed
                // - timeout reached
                Ok(Err(_)) | Err(_) | Ok(Ok(None)) => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(parent: &span, "receiver timed out");
                }
            }
        }

        #[cfg(feature = "tracing")]
        tracing::debug!(parent: &span, "bucket appears finished, removing");

        self.buckets
            .lock()
            .expect("ratelimit buckets poisoned")
            .remove(&self.path);
    }

    async fn handle_headers(&self, headers: &RatelimitHeaders) {
        let ratelimits = match headers {
            RatelimitHeaders::GlobalLimited { reset_after } => {
                self.lock_global(Duration::from_secs(*reset_after)).await;

                None
            }
            RatelimitHeaders::None => return,
            RatelimitHeaders::Present {
                global,
                limit,
                remaining,
                reset_after,
                ..
            } => {
                if *global {
                    self.lock_global(Duration::from_secs(*reset_after)).await;
                }

                Some((*limit, *remaining, *reset_after))
            }
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(path=?self.path, "updating bucket");
        self.bucket.update(ratelimits);
    }

    async fn lock_global(&self, wait: Duration) {
        #[cfg(feature = "tracing")]
        tracing::debug!(path=?self.path, "request got global ratelimited");
        self.global.lock();
        let lock = self.global.0.lock().await;
        sleep(wait).await;
        self.global.unlock();

        drop(lock);
    }

    async fn next(&self) -> Option<Sender<Sender<Option<RatelimitHeaders>>>> {
        #[cfg(feature = "tracing")]
        tracing::debug!(path=?self.path, "starting to get next in queue");

        self.wait_if_needed().await;

        self.bucket.queue.pop(Self::WAIT).await
    }

    async fn wait_if_needed(&self) {
        #[cfg(feature = "tracing")]
        let span = tracing::debug_span!("waiting for bucket to refresh", path=?self.path);

        let wait = {
            if self.bucket.remaining() > 0 {
                return;
            }

            #[cfg(feature = "tracing")]
            tracing::debug!(parent: &span, "0 tickets remaining, may have to wait");

            match self.bucket.time_remaining() {
                TimeRemaining::Finished => {
                    self.bucket.try_reset();

                    return;
                }
                TimeRemaining::NotStarted => return,
                TimeRemaining::Some(dur) => dur,
            }
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(
            parent: &span,
            milliseconds=%wait.as_millis(),
            "waiting for ratelimit to pass",
        );

        sleep(wait).await;

        #[cfg(feature = "tracing")]
        tracing::debug!(parent: &span, "done waiting for ratelimit to pass");

        self.bucket.try_reset();
    }
}
