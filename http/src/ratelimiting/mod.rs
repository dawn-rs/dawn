pub mod error;

mod bucket;
mod headers;

pub use self::{
    error::{RatelimitError, RatelimitResult},
    headers::RatelimitHeaders,
};

use self::bucket::{Bucket, BucketQueueTask, TimeRemaining};
use crate::routing::Path;
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{
    oneshot::{self, Receiver, Sender},
    Mutex as AsyncMutex,
};

/// Global lock. We use a pair to avoid actually locking the mutex every check.
/// This allows futures to only wait on the global lock when a global ratelimit
/// is in place by, in turn, waiting for a guard, and then each immediately
/// dropping it.
#[derive(Debug, Default)]
struct GlobalLockPair(AsyncMutex<()>, AtomicBool);

impl GlobalLockPair {
    pub fn lock(&self) {
        self.1.store(true, Ordering::Release);
    }

    pub fn unlock(&self) {
        self.1.store(false, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.1.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Ratelimiter {
    buckets: Arc<Mutex<HashMap<Path, Arc<Bucket>>>>,
    global: Arc<GlobalLockPair>,
}

impl Ratelimiter {
    /// Create a new ratelimiter.
    ///
    /// Most users won't need to use this directly. If you're creating your own
    /// HTTP proxy then this is good to use for your own ratelimiting.
    pub fn new() -> Self {
        Self::default()
    }

    #[deprecated(since = "0.5.0", note = "use `ticket` instead, which is not async")]
    pub async fn get(&self, path: Path) -> Receiver<Sender<Option<RatelimitHeaders>>> {
        self.ticket(path)
    }

    pub fn ticket(&self, path: Path) -> Receiver<Sender<Option<RatelimitHeaders>>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("getting bucket for path: {:?}", path);

        let (tx, rx) = oneshot::channel();
        let (bucket, fresh) = self.entry(path.clone(), tx);

        if fresh {
            tokio::spawn(
                BucketQueueTask::new(
                    bucket,
                    Arc::clone(&self.buckets),
                    Arc::clone(&self.global),
                    path,
                )
                .run(),
            );
        }

        rx
    }

    /// Provide an estimate for the time left until a path can be used
    /// without being ratelimited.
    ///
    /// This method is not guaranteed to be accurate and may return
    /// None if either no ratelimit is known or buckets are remaining.
    pub async fn time_until_available(&self, path: &Path) -> Option<Duration> {
        let buckets = self.buckets.lock().expect("ratelimit buckets poisoned");

        match buckets.get(path)?.time_remaining() {
            TimeRemaining::Finished | TimeRemaining::NotStarted => None,
            TimeRemaining::Some(duration) => Some(duration),
        }
    }

    fn entry(
        &self,
        path: Path,
        tx: Sender<Sender<Option<RatelimitHeaders>>>,
    ) -> (Arc<Bucket>, bool) {
        // nb: not realisically point of contention
        let mut buckets = self.buckets.lock().expect("ratelimit buckets poisoned");

        match buckets.entry(path.clone()) {
            Entry::Occupied(bucket) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("got existing bucket: {:?}", path);

                let bucket = bucket.into_mut();
                bucket.queue.push(tx);
                #[cfg(feature = "tracing")]
                tracing::debug!("added request into bucket queue: {:?}", path);

                (Arc::clone(&bucket), false)
            }
            Entry::Vacant(entry) => {
                #[cfg(feature = "tracing")]
                tracing::debug!("making new bucket for path: {:?}", path);
                let bucket = Bucket::new(path);
                bucket.queue.push(tx);

                let bucket = Arc::new(bucket);
                entry.insert(Arc::clone(&bucket));

                (bucket, true)
            }
        }
    }
}
